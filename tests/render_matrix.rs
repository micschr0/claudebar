//! Byte-exact render matrix — the regression net for refactors.
//!
//! Renders every fixture under every theme × style × layout variant with all
//! twelve segments enabled, and byte-compares the result against a committed
//! golden file. A refactor that claims to be behavior-preserving proves it here.
//!
//! This is deliberately broader than `render_golden.rs`, which covers only the
//! eight default segments under one style pair. The gaps that matrix left —
//! `burn`, `clock`, `dev-context`, `update-notice`, the `auto` wrap path, and
//! five of the eight styles — are exactly where a silent regression could hide.
//!
//! Regenerate: `UPDATE_MATRIX=1 cargo test --test render_matrix`
//!
//! Runs in the normal suite (~5s). See [`build_matrix`] for why it is 544 rows
//! and not the 7168 of the full cross product.
//!
//! # Determinism
//!
//! Every ambient input is pinned, because each one silently varies per machine
//! or per run otherwise:
//!
//! - `now`, `home`, `tz_offset` — passed to `render_with` directly.
//! - `clock_mode` — pinned per variant. Left on `"auto"` it reads `LC_TIME` /
//!   `LC_ALL` / `LANG` and picks 12h or 24h from the *machine's locale*.
//! - `COLUMNS` — pinned per variant. Unset, `render_auto` shells out to `stty`
//!   and the wrap point depends on the terminal running the test. Worse: the
//!   widest line here is 128 columns, so a generous width makes the wrap path
//!   silently never execute.
//! - `CLAUDEBAR_BURN_FILE` — seeded read-only (see `seed_burn_file`).
//! - `XDG_CONFIG_HOME` — redirected so the `update-notice` badge reads a seeded
//!   cache instead of whatever this machine happens to have.
//! - `CLAUDEBAR_LIMIT_SYNC_DIR` — redirected; the store is off by default but a
//!   stray real one must not leak in.
//!
//! Two preconditions cannot be pinned, only asserted — see `check_preconditions`.

use claudebar::model::{Config, InputData, SegmentKind};
use claudebar::render::render_with;
use claudebar::{styles, themes};
use std::fs;
use std::path::{Path, PathBuf};

/// Just before the fixtures' far-future `resets_at` epochs, so every countdown
/// is positive and stable. Matches `render_golden.rs`.
const FIXED_NOW: i64 = 1_899_990_000;
const FIXED_HOME: &str = "/home/me";
/// UTC+1 — non-zero on purpose, so a bug that drops the offset shows up.
const FIXED_TZ: i32 = 3600;

const GOLDEN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/golden/render_matrix.txt"
);

/// `(layout, COLUMNS, clock_mode)`.
///
/// Both `auto` widths are below the 128-column widest line on purpose: that is
/// what forces the wrap path to actually run.
const VARIANTS: &[(&str, &str, &str)] = &[
    ("fixed", "0", "24h"),
    ("fixed", "0", "12h"),
    ("auto", "60", "24h"),
    ("auto", "100", "24h"),
];

/// The theme the structure sweep runs on. Any theme works — it fills the same
/// slots — so the default is the honest pick.
const STRUCTURE_THEME: &str = "tokyo-night";

/// Fixtures for the palette sweep, chosen to light up as many colour slots as
/// possible between them: `typical` covers model/context/rate-limits/cost,
/// `dev_context` adds worktree/PR/agent, `over_limit_5h` drives the crit colours.
const PALETTE_FIXTURES: &[&str] = &["typical", "dev_context", "over_limit_5h"];

/// One icon-ful and one icon-less style. `SegmentWriter::icon` is a no-op when
/// `icons` is off, which is its own rendering path worth pinning per theme.
const PALETTE_STYLES: &[&str] = &["powerline", "ascii"];

#[test]
fn render_matrix_is_byte_identical() {
    let tmp = unique_temp_dir();
    fs::create_dir_all(&tmp).expect("temp dir");

    let burn_file = tmp.join("burn-5h.tsv");
    let burn_seed = seed_burn_file(&burn_file);
    seed_update_cache(&tmp);

    // SAFETY: this file holds exactly one `#[test]`, so the test binary is
    // single-threaded here and nothing else reads the environment concurrently.
    unsafe {
        std::env::set_var("CLAUDEBAR_BURN_FILE", &burn_file);
        std::env::set_var("CLAUDEBAR_LIMIT_SYNC_DIR", tmp.join("limit-sync"));
        std::env::set_var("XDG_CONFIG_HOME", &tmp);
        // Belt and braces: `clock_mode` is pinned per variant, but this keeps
        // the locale out of the picture entirely.
        std::env::set_var("LC_ALL", "C");
    }

    check_preconditions();

    let actual = build_matrix();

    // The seeded burn file must come out exactly as it went in. If it changed,
    // the read-only guard failed and every burn row is order-dependent noise
    // rather than a fixed projection — the golden would be unreproducible.
    let burn_after = fs::read_to_string(&burn_file).expect("burn file readable");
    assert_eq!(
        burn_after, burn_seed,
        "burn sample file was written during the run: the read-only guard failed, \
         so burn output depends on render order and this golden is not reproducible"
    );

    let _ = fs::remove_dir_all(&tmp);

    if std::env::var_os("UPDATE_MATRIX").is_some() {
        fs::create_dir_all(Path::new(GOLDEN).parent().unwrap()).expect("golden dir");
        fs::write(GOLDEN, &actual).expect("write golden");
        eprintln!("matrix golden updated: {} rows", actual.lines().count());
        return;
    }

    let expected = fs::read_to_string(GOLDEN).unwrap_or_else(|e| {
        panic!(
            "golden missing ({e}); regenerate with UPDATE_MATRIX=1 cargo test --test render_matrix"
        )
    });

    if actual != expected {
        panic!("{}", describe_diff(&expected, &actual));
    }
}

/// Build the table as two sweeps rather than one full cross product.
///
/// The naive `fixtures × themes × styles × variants` product is 7168 rows and
/// 3 MB, and almost all of it is redundant: a theme contributes *only* colour
/// slots, never structure, so re-rendering all 14 fixtures under all 8 styles
/// once per theme re-proves the same 16 palettes 112 times over. The cost is
/// not disk, it is that a 7168-row diff cannot be reviewed — and reviewing the
/// diff is the entire point when a change is *meant* to alter output.
///
/// Splitting the axes keeps the defect-detection power and drops the row count
/// by 14×:
///
/// - **Structure sweep** — every fixture × style × variant on one theme. Catches
///   anything that changes layout, glyphs, segment order, or wrapping.
/// - **Palette sweep** — every theme on the segment-richest fixtures, in an
///   icon-ful and an icon-less style. Catches a swapped or drifted colour slot.
fn build_matrix() -> String {
    let fixtures = load_fixtures();
    let mut out = String::with_capacity(1 << 18);

    // ── Structure sweep ──────────────────────────────────────────────────────
    for &(layout, columns, clock_mode) in VARIANTS {
        // SAFETY: single-threaded (see the caller's note). Set once per
        // variant rather than per render — `render_auto` reads it every call.
        unsafe { std::env::set_var("COLUMNS", columns) };

        for (name, json) in &fixtures {
            let input = InputData::parse(json);
            for &style_name in styles::NAMES {
                out.push_str(&row(
                    &input,
                    name,
                    STRUCTURE_THEME,
                    style_name,
                    layout,
                    columns,
                    clock_mode,
                ));
            }
        }
    }

    // ── Palette sweep ────────────────────────────────────────────────────────
    // SAFETY: as above.
    unsafe { std::env::set_var("COLUMNS", "0") };
    for (name, json) in &fixtures {
        if !PALETTE_FIXTURES.contains(&name.as_str()) {
            continue;
        }
        let input = InputData::parse(json);
        for &theme_name in themes::NAMES {
            for &style_name in PALETTE_STYLES {
                out.push_str(&row(
                    &input, name, theme_name, style_name, "fixed", "0", "24h",
                ));
            }
        }
    }

    out
}

/// One table row. Column order is fixed, so a diff points straight at the
/// combination that changed.
#[allow(clippy::too_many_arguments)]
fn row(
    input: &InputData,
    fixture: &str,
    theme_name: &str,
    style_name: &str,
    layout: &str,
    columns: &str,
    clock_mode: &str,
) -> String {
    let mut cfg = Config {
        theme: theme_name.to_string(),
        style: style_name.to_string(),
        segments: SegmentKind::ALL.to_vec(),
        ..Default::default()
    };
    cfg.thresholds.layout = layout.to_string();
    cfg.thresholds.clock_mode = clock_mode.to_string();

    let theme = themes::get(theme_name);
    let style = styles::get(style_name);
    let line = render_with(
        input,
        &cfg,
        &theme,
        &style,
        FIXED_NOW,
        Some(FIXED_HOME),
        FIXED_TZ,
    );

    format!(
        "{layout}/{columns}/{clock_mode}|{fixture}|{theme_name}|{style_name}|{}\n",
        escape(&line)
    )
}

/// Every `fixtures/*.json`, sorted by name so row order never depends on
/// directory iteration order.
fn load_fixtures() -> Vec<(String, String)> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures");
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .expect("fixtures dir")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "no fixtures found in {dir}");

    let loaded: Vec<(String, String)> = paths
        .iter()
        .map(|p| {
            let name = p.file_stem().unwrap().to_string_lossy().into_owned();
            (name, fs::read_to_string(p).expect("read fixture"))
        })
        .collect();

    // A typo in PALETTE_FIXTURES would silently skip the whole palette sweep
    // and still produce a green, smaller golden.
    for want in PALETTE_FIXTURES {
        assert!(
            loaded.iter().any(|(name, _)| name == want),
            "PALETTE_FIXTURES names {want:?}, which is not in fixtures/"
        );
    }

    loaded
}

/// One table row per rendered line. ESC becomes `\e` and the wrap newline
/// becomes `\n` so each combination stays exactly one line in the golden — a
/// raw newline would silently split one row into two and misalign every diff.
fn escape(line: &str) -> String {
    line.replace('\x1b', "\\e").replace('\n', "\\n")
}

/// Seed the burn sample file, then drop write permission.
///
/// Both halves matter. Without the seed, `read_samples` finds nothing and every
/// burn row renders the same `warming` placeholder, leaving the projection
/// arithmetic untested. Without the read-only bit, `Burn::render` appends a
/// sample on *every* one of the thousands of renders, so each row depends on
/// how many ran before it.
///
/// Five samples 100s apart rising 10%→50% give a clean +0.1 %/s slope, which
/// puts the segment in its `active` state with a computable ETA.
fn seed_burn_file(path: &Path) -> String {
    let seed: String = (1..=5u32)
        .map(|i| {
            format!(
                "{}\t{:.3}\t{}\n",
                FIXED_NOW - 600 + i64::from(i) * 100,
                f64::from(i) * 10.0,
                FIXED_NOW + 3600
            )
        })
        .collect();
    fs::write(path, &seed).expect("seed burn file");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o444)).expect("chmod burn file");
    }

    seed
}

/// Seed the update-check cache with an unreachably high version so the
/// `update-notice` badge renders in every row. Left unseeded the segment emits
/// nothing and the badge path goes untested.
fn seed_update_cache(xdg_config_home: &Path) {
    let dir = xdg_config_home.join("claudebar");
    fs::create_dir_all(&dir).expect("config dir");
    fs::write(
        dir.join("update-check.json"),
        format!(r#"{{"checked_at":{FIXED_NOW},"latest":"2999.1.1"}}"#),
    )
    .expect("seed update cache");
}

/// Assert the two things this golden depends on that cannot be pinned from
/// inside the test. Both are silent-wrong-answer failures otherwise: the matrix
/// would still pass, just against different content than it was recorded with.
fn check_preconditions() {
    // `fixtures/no_git.json` points `cwd` at `/tmp`, which exists — so the git
    // segment really shells out there. The golden was recorded on a machine
    // where /tmp is not a repository and the segment stays empty.
    let in_repo = std::process::Command::new("git")
        .args(["-C", "/tmp", "rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    assert!(
        !in_repo,
        "/tmp is inside a git repository on this machine, so fixtures/no_git.json \
         renders a git segment that the golden was not recorded with"
    );

    // The update badge only shows a release newer than the running binary. If
    // claudebar ever ships a version above the seed, the badge vanishes and
    // every row changes for a reason that has nothing to do with rendering.
    let running =
        claudebar::update::Version::parse(env!("CARGO_PKG_VERSION")).expect("own version parses");
    let seeded = claudebar::update::Version::parse("2999.1.1").unwrap();
    assert!(
        seeded > running,
        "seeded update version is no longer newer than the running binary"
    );
}

/// Report the first differing row plus a total count, rather than dumping
/// thousands of lines of diff into the test output.
fn describe_diff(expected: &str, actual: &str) -> String {
    let exp: Vec<&str> = expected.lines().collect();
    let act: Vec<&str> = actual.lines().collect();
    let changed =
        exp.iter().zip(act.iter()).filter(|(a, b)| a != b).count() + exp.len().abs_diff(act.len());

    let first = exp
        .iter()
        .zip(act.iter())
        .enumerate()
        .find(|(_, (a, b))| a != b);

    let detail = match first {
        Some((i, (e, a))) => format!(
            "first difference at row {}:\n  expected: {}\n  actual:   {}",
            i + 1,
            truncate(e),
            truncate(a)
        ),
        None => format!(
            "row count differs: golden {} vs actual {}",
            exp.len(),
            act.len()
        ),
    };

    format!(
        "render matrix changed: {changed} of {} rows differ.\n{detail}\n\n\
         If this change is intended, review it row by row, then regenerate:\n  \
         UPDATE_MATRIX=1 cargo test --test render_matrix",
        exp.len().max(act.len())
    )
}

fn truncate(s: &str) -> String {
    if s.chars().count() <= 240 {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(240).collect::<String>())
    }
}

/// Unique temp dir; pid + nanos keep parallel runs from colliding. No
/// `tempfile` crate — `insta` is the only dev-dependency.
fn unique_temp_dir() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("claudebar-matrix-{}-{}", std::process::id(), nanos))
}
