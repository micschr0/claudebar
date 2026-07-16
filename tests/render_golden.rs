//! Full-line golden snapshots: render every fixture under the default config
//! (Tokyo Night + Powerline) with a fixed clock and a fixed `$HOME`, so the
//! output is deterministic. ESC bytes are rendered as `\e` for readable diffs.
//!
//! Update snapshots with `cargo insta review` (or `INSTA_UPDATE=always`).
//!
//! Note: fixtures whose `cwd` points at a non-existent path produce no git
//! segment (the git command fails), which keeps these snapshots independent of
//! the checkout's own git state.

use claudebar::model::{Config, InputData};
use claudebar::render::render_with;
use claudebar::{styles, themes};

/// Just before the fixtures' far-future `resets_at` epochs, so countdowns are
/// present and stable.
const FIXED_NOW: i64 = 1_899_990_000;

fn render_fixture(json: &str) -> String {
    let input = InputData::parse(json);
    let cfg = Config::default();
    let theme = themes::get(&cfg.theme);
    let style = styles::get(&cfg.style);
    let line = render_with(&input, &cfg, &theme, &style, FIXED_NOW, Some("/home/me"), 0);
    // Readable, and proves no raw ESC from host strings leaks through (every
    // ESC in the output is one we emitted as a color code).
    line.replace('\x1b', "\\e")
}

#[test]
fn golden_lines() {
    insta::glob!(env!("CARGO_MANIFEST_DIR"), "fixtures/*.json", |path| {
        let json = std::fs::read_to_string(path).unwrap();
        insta::assert_snapshot!(render_fixture(&json));
    });
}

/// Render one combo of theme + style against `json`, ESC → `\e` for readable
/// diffs. Mirrors `render_fixture` but parametrizes theme and style names.
fn render_combo(json: &str, theme_name: &str, style_name: &str) -> String {
    let input = InputData::parse(json);
    let cfg = Config {
        theme: theme_name.to_string(),
        style: style_name.to_string(),
        ..Default::default()
    };
    let theme = themes::get(theme_name);
    let style = styles::get(style_name);
    let line = render_with(&input, &cfg, &theme, &style, FIXED_NOW, Some("/home/me"), 0);
    line.replace('\x1b', "\\e")
}

/// CR-12: full theme × {ascii, powerline} golden matrix against
/// `fixtures/typical.json`. 16 themes × 2 styles = 32 snapshots, each under a
/// distinct `{name}__{style}` suffix, so none collide with the `golden_lines`
/// glob outputs. `typical.json`'s `cwd` is a non-existent path, so no git
/// subprocess runs and the snapshots stay independent of the checkout state.
#[test]
fn golden_matrix() {
    let json = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/typical.json"
    ))
    .unwrap();
    for &name in themes::NAMES {
        for &style in &["ascii", "powerline"] {
            let rendered = render_combo(&json, name, style);
            insta::with_settings!({ snapshot_suffix => format!("{name}__{style}") }, {
                insta::assert_snapshot!(rendered);
            });
        }
    }
}

/// CR-15: prove no host-supplied control byte leaks end-to-end. Render
/// `fixtures/injection.json` (its `cwd` and `model.display_name` carry ESC/BEL/
/// CR/LF), strip only the renderer's own SGR runs (`\x1b[...m`), then assert the
/// residue contains none of ESC/BEL/CR/LF — the explicit version of the
/// `render_fixture` informal comment, anchoring the `sanitize::strip_control`
/// contract.
#[test]
fn injection_no_control_byte_leak() {
    let json = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/injection.json"
    ))
    .unwrap();
    let input = InputData::parse(&json);
    let cfg = Config::default();
    let theme = themes::get(&cfg.theme);
    let style = styles::get(&cfg.style);
    let rendered = render_with(&input, &cfg, &theme, &style, FIXED_NOW, Some("/home/me"), 0);

    // Strip the renderer's own SGR sequences (`\x1b[` … `m`) by hand, so we do
    // not take a direct dependency on the `regex` crate.
    let mut residue = String::with_capacity(rendered.len());
    let mut chars = rendered.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next(); // consume '['
            // Skip the `[0-9;]*` parameter run, then the terminating 'm'.
            for p in chars.by_ref() {
                if p == 'm' {
                    break;
                }
            }
            continue;
        }
        residue.push(c);
    }

    for (byte, label) in [('\x1b', "ESC"), ('\x07', "BEL"), ('\r', "CR"), ('\n', "LF")] {
        assert!(
            !residue.contains(byte),
            "{label} control byte leaked into output: residue={residue:?}"
        );
    }
}

#[test]
fn render_fixed_emits_full_pipeline() {
    // Exercise the full default segment pipeline: directory, git, model,
    // context, lines, rate-limits, cost, duration — with a far-future
    // resets_at so countdowns are positive and stable.
    let json = r#"{
        "cwd": "/home/me/projects/claude-code",
        "model": { "display_name": "claude-opus-4-5" },
        "context_window": {
            "total_input_tokens": 35000,
            "total_output_tokens": 7300,
            "used_percentage": 67.0
        },
        "rate_limits": {
            "five_hour": { "used_percentage": 48.0, "resets_at": 1900000000 }
        },
        "cost": { "total_cost_usd": 0.50, "total_lines_added": 10 },
        "effort": { "level": "high" }
    }"#;
    insta::assert_snapshot!(render_fixture(json));
}
/// Maximum-coverage fixture: all segments populated with highest reasonable
/// values (model="claude-opus-4-5", cost=$9.99, lines +50/-10, context heavy,
/// rate-limits at 80%, effort max, approved review, sub-agent active).
#[test]
fn golden_ultra_effort() {
    let json = r#"{
        "cwd": "/home/me/projects/claude-code",
        "model": { "display_name": "claude-opus-4-5" },
        "context_window": {
            "total_input_tokens": 120000,
            "total_output_tokens": 30000,
            "used_percentage": 95.0
        },
        "rate_limits": {
            "five_hour": { "used_percentage": 80.0, "resets_at": 1900000000 },
            "seven_day": { "used_percentage": 55.0, "resets_at": 1900100000 }
        },
        "cost": {
            "total_cost_usd": 9.99,
            "total_lines_added": 50,
            "total_lines_removed": 10,
            "total_duration_ms": 3600000
        },
        "effort": { "level": "max" },
        "pr": { "number": 42, "review_state": "approved" },
        "agent": { "name": "experimenter" },
        "worktree": { "name": "feat-ultra" }
    }"#;
    insta::assert_snapshot!(render_fixture(json));
}
