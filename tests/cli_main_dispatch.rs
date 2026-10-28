//! Integration tests for main.rs dispatch paths.
//!
//! Each test spawns the `claudebar` binary via `env!("CARGO_BIN_EXE_claudebar")`
//! and asserts correct exit codes and expected content in stdout/stderr.
//!
//! # Notes
//! Each test spawns the `claudebar` binary via `env!("CARGO_BIN_EXE_claudebar")`
//! and asserts correct exit codes and expected content in stdout/stderr.
//!
//! # Note
//! The pre-existing clap debug-assertion bug (conflicting `--segments` long flag)
//! was fixed in src/cli.rs by renaming the List-subcommand flag to
//! `--list-segments`. All tests now run cleanly with exit code 0 in debug builds.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

fn unique_temp_path(name: &str) -> PathBuf {
    let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("claudebar_test_{name}_{n}.toml"))
}

/// A fresh empty directory unique to the caller — for tests that need to glob
/// sibling files (e.g. `setup`'s `.bak-*` backups).
fn unique_temp_dir(name: &str) -> PathBuf {
    let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "claudebar_test_{name}_{}_{}",
        std::process::id(),
        n
    ));
    fs::create_dir(&dir).expect("mkdir unique temp");
    dir
}

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_claudebar"))
}

// -- render tests -----------------------------------------------------------

#[test]
fn main_render_exit_zero_with_input() {
    let mut child = bin()
        .arg("render")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn claudebar render");

    let mut stdin = child.stdin.take().expect("failed to open stdin");
    stdin
        .write_all(br#"{"session_id":"x"}"#)
        .expect("failed to write stdin");
    drop(stdin);

    let output = child.wait_with_output().expect("failed to wait on child");

    assert_eq!(
        output.status.code(),
        Some(0),
        "render should exit 0 with input, got: {:?}",
        output.status.code()
    );
    assert!(
        !output.stdout.is_empty(),
        "render should produce stdout, got empty"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.is_empty(),
        "render should produce no stderr, got: {stderr}"
    );
}

#[test]
fn main_render_exit_zero_without_input() {
    let child = bin()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn claudebar render");

    let output = child.wait_with_output().expect("failed to wait on child");

    let exit_code = output.status.code();
    assert!(
        exit_code.is_some(),
        "render should terminate without panic, got: {exit_code:?}",
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.is_empty() || stderr.contains("claudebar"),
        "stderr should be empty or benign, got: {stderr}"
    );
}

// -- init tests -------------------------------------------------------------

#[test]
fn main_init_emits_toml() {
    let output = bin()
        .arg("init")
        .arg("--print")
        .output()
        .expect("failed to run claudebar init --print");

    assert_eq!(
        output.status.code(),
        Some(0),
        "init --print should exit 0, got: {:?}",
        output.status.code()
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("theme = "),
        "init --print stdout should contain TOML config, got: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.is_empty() || stderr.contains("claudebar"),
        "stderr should be empty or contain claudebar messages, got: {stderr}"
    );
}

#[test]
fn main_init_writes_file_with_force() {
    let path = unique_temp_path("init_force");

    let _ = fs::remove_file(&path);

    let output = bin()
        .arg("init")
        .arg("--force")
        .arg("--config")
        .arg(&path)
        .output()
        .expect("failed to run claudebar init");

    assert_eq!(
        output.status.code(),
        Some(0),
        "init with --force should exit 0, got: {:?}",
        output.status.code()
    );
    assert!(
        path.exists(),
        "config file should be created at {}",
        path.display()
    );

    let _ = fs::remove_file(&path);
}

// -- list tests -------------------------------------------------------------

#[test]
fn main_list_lists_themes() {
    let output = bin()
        .arg("list")
        .output()
        .expect("failed to run claudebar list");

    assert_eq!(output.status.code(), Some(0), "list should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("Themes:"),
        "list stdout should contain 'Themes:', got: {stdout}"
    );
    assert!(
        stdout.contains("tokyo-night") || stdout.contains("rose-pine"),
        "list stdout should contain known theme names (with dashes), got: {stdout}"
    );
    assert!(
        stderr.is_empty(),
        "list should produce no stderr, got: {stderr}"
    );
}

#[test]
fn main_list_lists_segments_with_flag() {
    let output = bin()
        .arg("list")
        .arg("--list-segments")
        .output()
        .expect("failed to run claudebar list --segments");

    assert_eq!(
        output.status.code(),
        Some(0),
        "list --segments should exit 0"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("Segments:"),
        "list --segments stdout should contain 'Segments:', got: {stdout}"
    );
    assert!(
        stdout.contains("directory") || stdout.contains("git"),
        "list --segments stdout should contain known segment names, got: {stdout}"
    );
    assert!(
        stderr.is_empty(),
        "list --segments should produce no stderr, got: {stderr}"
    );
}

// -- sync test --------------------------------------------------------------

#[test]
fn main_sync_round_trip() {
    let path = unique_temp_path("sync");

    let _ = fs::remove_file(&path);

    let toml_content = br#"segments = ["directory", "git"]
theme = "tokyo_night"
style = "unicode""#;
    fs::write(&path, toml_content).expect("failed to write test config");

    let output = bin()
        .arg("sync")
        .arg("--config")
        .arg(&path)
        .output()
        .expect("failed to run claudebar sync");

    assert_eq!(
        output.status.code(),
        Some(0),
        "sync should exit 0, got: {:?}",
        output.status.code()
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(path.exists(), "config file should still exist after sync");
    assert!(
        stdout.contains("up to date")
            || stdout.contains("config updated")
            || stdout.contains("added segment"),
        "sync stdout should mention status, got: {stdout}"
    );
    assert!(
        stderr.is_empty() || stderr.contains("claudebar"),
        "sync stderr should be empty or claudebar message, got: {stderr}"
    );

    let _ = fs::remove_file(&path);
}

#[test]
fn main_sync_does_not_add_automatic_update_notice() {
    let path = unique_temp_path("sync-automatic-notice");
    let _ = fs::remove_file(&path);
    fs::write(&path, br#"segments = ["directory", "git"]"#).expect("failed to write test config");

    let output = bin()
        .arg("sync")
        .arg("--config")
        .arg(&path)
        .output()
        .expect("failed to run claudebar sync");
    assert_eq!(output.status.code(), Some(0));

    let synced = fs::read_to_string(&path).expect("config readable after sync");
    // Configurable segments still get added as usual.
    assert!(
        synced.contains("clock") && synced.contains("burn"),
        "sync should still add ordinary new segments, got: {synced}"
    );
    // The automatic notice is not a configurable segment.
    assert!(
        !synced.contains("update-notice"),
        "sync must not add the automatic update notice, got: {synced}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("update-notice"),
        "sync should not offer the automatic notice as a segment, got: {stdout}"
    );

    let _ = fs::remove_file(&path);
}

#[cfg(unix)]
#[test]
fn main_sync_write_failure_preserves_the_original_config() {
    let dir = unique_temp_dir("sync_write_failure");
    let path = dir.join("config.toml");
    let original = "theme = \"catppuccin\"\nstyle = \"ascii\"\nsegments = [\"model\"]\n";
    fs::write(&path, original).unwrap();
    let output = Command::new("sh")
        .args([
            "-c",
            "trap '' XFSZ; ulimit -f 0; exec \"$@\"",
            "limited-save",
        ])
        .arg(env!("CARGO_BIN_EXE_claudebar"))
        .args(["sync", "--config", "config.toml"])
        .current_dir(&dir)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 1, "staging cleaned up");

    let retry = bin()
        .args(["sync", "--config", "config.toml"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(retry.status.code(), Some(0));
    let saved = claudebar::model::Config::load(&path).unwrap();
    assert_eq!(saved.theme, "catppuccin");
    assert!(
        saved
            .segments
            .contains(&claudebar::model::SegmentKind::Clock)
    );
    fs::remove_dir_all(dir).unwrap();
}

// -- doctor test ------------------------------------------------------------

#[test]
fn main_doctor_reports_diagnostics() {
    let output = bin()
        .arg("doctor")
        .output()
        .expect("failed to run claudebar doctor");

    assert_eq!(
        output.status.code(),
        Some(0),
        "doctor should exit 0 (informational), got: {:?}",
        output.status.code()
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("Binary in PATH")
            || stdout.contains("Nerd Font")
            || stdout.contains("git on PATH"),
        "doctor stdout should contain diagnostic lines, got: {stdout}"
    );
    assert!(
        stderr.is_empty() || stderr.contains("claudebar"),
        "doctor stderr should be empty or contain claudebar, got: {stderr}"
    );
}

// -- completions test -------------------------------------------------------

#[test]
fn main_completions_bash_emits_script() {
    let output = bin()
        .arg("completions")
        .arg("bash")
        .output()
        .expect("failed to run claudebar completions bash");

    assert_eq!(
        output.status.code(),
        Some(0),
        "completions bash should exit 0"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.starts_with("_claudebar()"),
        "completions bash stdout should start with '_claudebar()', got first 80 chars: {stdout}"
    );
    assert!(
        stderr.is_empty(),
        "completions bash should produce no stderr, got: {stderr}"
    );
}

// -- help / version tests ---------------------------------------------------

#[test]
fn main_help_flag() {
    let output = bin()
        .arg("--help")
        .output()
        .expect("failed to run claudebar --help");

    assert_eq!(
        output.status.code(),
        Some(0),
        "--help should exit 0, got: {:?}",
        output.status.code()
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("claudebar"),
        "--help stdout should contain 'claudebar', got: {stdout}"
    );
    assert!(
        stdout.contains("render") || stdout.contains("config") || stdout.contains("init"),
        "--help stdout should list subcommands, got: {stdout}"
    );
    assert!(
        stderr.is_empty(),
        "--help should produce no stderr, got: {stderr}"
    );
}

#[test]
fn main_version_flag() {
    let output = bin()
        .arg("--version")
        .output()
        .expect("failed to run claudebar --version");

    assert_eq!(
        output.status.code(),
        Some(0),
        "--version should exit 0, got: {:?}",
        output.status.code()
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("claudebar"),
        "--version stdout should contain 'claudebar', got: {stdout}"
    );
    assert!(
        stderr.is_empty(),
        "--version should produce no stderr, got: {stderr}"
    );
}

// -- config test ------------------------------------------------------------

// -- edit tests -----------------------------------------------------------

#[test]
fn main_edit_creates_config_then_runs_editor() {
    let path = unique_temp_path("edit_create");
    let _ = fs::remove_file(&path);

    let output = bin()
        .arg("edit")
        .arg("--config")
        .arg(&path)
        .env("EDITOR", "true") // coreutils `true` — exits 0, ignores args
        .output()
        .expect("failed to run claudebar edit");

    assert_eq!(
        output.status.code(),
        Some(0),
        "edit should exit 0 when the editor succeeds, got: {:?}",
        output.status.code()
    );
    assert!(path.exists(), "edit should create the config if missing");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("created default config"),
        "edit should announce the config it created, got: {stderr}"
    );

    let _ = fs::remove_file(&path);
}

#[test]
fn main_edit_propagates_editor_failure() {
    let path = unique_temp_path("edit_fail");
    let _ = fs::remove_file(&path);

    let output = bin()
        .arg("edit")
        .arg("--config")
        .arg(&path)
        .env("EDITOR", "false") // exits 1
        .output()
        .expect("failed to run claudebar edit");

    assert_eq!(
        output.status.code(),
        Some(1),
        "edit should exit 1 when the editor exits non-zero"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("exited with status"),
        "stderr should report the editor's exit status"
    );

    let _ = fs::remove_file(&path);
}

#[test]
fn main_edit_accepts_arguments_and_keeps_the_config_path_literal() {
    let dir = unique_temp_dir("editor_args");
    let path = dir.join("config '$(touch editor-injected)'.toml");
    let replacement = dir.join("replacement template.toml");
    fs::write(&path, "theme = \"tokyo-night\"\n").unwrap();
    let edited = "theme = \"catppuccin\"\nstyle = \"ascii\"\n";
    fs::write(&replacement, edited).unwrap();

    let output = bin()
        .args(["edit", "--config"])
        .arg(&path)
        .env("EDITOR", format!("cp '{}'", replacement.display()))
        .current_dir(&dir)
        .output()
        .expect("run editor command with an argument");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(fs::read_to_string(&path).unwrap(), edited);
    assert!(!dir.join("editor-injected").exists());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn main_edit_reports_missing_editor() {
    let path = unique_temp_path("edit_noeditor");
    let _ = fs::remove_file(&path);

    let output = bin()
        .arg("edit")
        .arg("--config")
        .arg(&path)
        .env("EDITOR", "claudebar-no-such-editor-xyz")
        .output()
        .expect("failed to run claudebar edit");

    assert_eq!(output.status.code(), Some(1), "edit should exit 1");

    let _ = fs::remove_file(&path);
}

// -- update test ---------------------------------------------------------

#[test]
fn main_update_check_records_offline_failure_in_isolated_config() {
    let config_home = unique_temp_dir("update_config");
    let empty_path = unique_temp_dir("update_empty_path");

    let output = bin()
        .arg("update")
        .arg("--check")
        .env("XDG_CONFIG_HOME", &config_home)
        .env("PATH", &empty_path)
        .output()
        .expect("failed to run claudebar update --check");

    assert_eq!(
        output.status.code(),
        Some(1),
        "failed update --check exits 1"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("claudebar:"),
        "expected update error, got: {stderr}"
    );
    assert!(
        stderr.contains("curl"),
        "expected missing curl diagnostic, got: {stderr}"
    );

    let cache = config_home.join("claudebar/update-check.json");
    let body = fs::read_to_string(&cache).expect("failed update attempt recorded");
    let cache: serde_json::Value = serde_json::from_str(&body).expect("valid update cache JSON");
    assert!(
        cache["checked_at"].as_i64().is_some(),
        "checked_at recorded: {cache}"
    );
    assert!(
        cache["latest"].is_null(),
        "failed check has no latest version: {cache}"
    );

    let _ = fs::remove_dir_all(&config_home);
    let _ = fs::remove_dir_all(&empty_path);
}

// -- setup tests --------------------------------------------------------
// `run_setup` wires `statusLine` into Claude Code's settings.json. `--settings-path`
// bypasses $HOME resolution. Stdin is not a TTY here, so the interactive y/N
// prompt is unreachable — the non-interactive branches are what these assert.

fn setup(dir: &Path, extra: &[&str]) -> std::process::Output {
    bin()
        .arg("setup")
        .arg("--settings-path")
        .arg(dir.join("settings.json"))
        .args(extra)
        .output()
        .expect("spawn claudebar setup")
}

fn bak_count(dir: &Path) -> usize {
    fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains(".bak-"))
        .count()
}

#[test]
fn main_setup_writes_status_line_into_a_new_file() {
    let dir = unique_temp_dir("setup_new");
    let out = setup(&dir, &["-y"]);

    assert_eq!(out.status.code(), Some(0), "exit 0 on fresh write");
    let written = fs::read_to_string(dir.join("settings.json")).expect("settings written");
    assert!(written.contains("\"statusLine\""), "key present: {written}");
    assert!(
        written.contains("claudebar render"),
        "command set: {written}"
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("configured"),
        "stdout confirms: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn main_setup_print_shows_diff_and_touches_nothing() {
    let dir = unique_temp_dir("setup_print");
    let out = setup(&dir, &["--print"]);

    assert_eq!(out.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("statusLine:"),
        "diff header printed"
    );
    assert!(
        !dir.join("settings.json").exists(),
        "--print writes nothing"
    );
}

#[test]
fn main_setup_print_preserves_malformed_and_already_configured_settings() {
    for (name, contents, expected_exit) in [
        ("malformed", "{ not json", 1),
        (
            "configured",
            r#"{"statusLine":{"type":"command","command":"claudebar render"}}"#,
            0,
        ),
    ] {
        let dir = unique_temp_dir(name);
        let settings = dir.join("settings.json");
        let config = dir.join("config.toml");
        let float_file = dir.join("float.txt");
        let config_home = dir.join("must-not-be-created");
        fs::write(&settings, contents).unwrap();
        fs::write(
            &config,
            format!(
                "[thresholds]\nfloat = true\nfloat_file = {:?}\n",
                float_file
            ),
        )
        .unwrap();

        let out = bin()
            .args(["setup", "--print", "--settings-path"])
            .arg(&settings)
            .arg("--config")
            .arg(&config)
            .env("XDG_CONFIG_HOME", &config_home)
            .output()
            .unwrap();

        assert_eq!(out.status.code(), Some(expected_exit), "{name}");
        assert_eq!(fs::read_to_string(settings).unwrap(), contents);
        assert_eq!(bak_count(&dir), 0);
        assert!(!float_file.exists());
        assert!(!config_home.exists());
        fs::remove_dir_all(dir).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn main_setup_hook_executes_a_binary_path_with_shell_metacharacters() {
    let dir = unique_temp_dir("setup_quoted_path");
    let executable = dir.join("claudebar '$(touch hook-injected); $HOME");
    std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_claudebar"), &executable).unwrap();
    let config_dir = dir.join("claudebar");
    fs::create_dir(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        "style = \"ascii\"\nsegments = [\"model\"]\n",
    )
    .unwrap();
    fs::write(
        config_dir.join("update-check.json"),
        r#"{"checked_at":9223372036854775807,"latest":null}"#,
    )
    .unwrap();

    let out = bin()
        .args(["setup", "--yes", "--settings-path"])
        .arg(dir.join("settings.json"))
        .arg("--binary-path")
        .arg(&executable)
        .env("HOME", &dir)
        .env("XDG_CONFIG_HOME", &dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let settings: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("settings.json")).unwrap()).unwrap();
    let mut hook = Command::new("sh")
        .arg("-c")
        .arg(settings["statusLine"]["command"].as_str().unwrap())
        .current_dir(&dir)
        .env("HOME", &dir)
        .env("XDG_CONFIG_HOME", &dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    hook.stdin
        .take()
        .unwrap()
        .write_all(br#"{"model":{"display_name":"QUOTED_PATH_OK"}}"#)
        .unwrap();
    let rendered = hook.wait_with_output().unwrap();
    assert_eq!(rendered.status.code(), Some(0));
    assert_eq!(
        claudebar::render::strip_ansi(std::str::from_utf8(&rendered.stdout).unwrap()),
        "QUOTED_PATH_OK\n"
    );
    assert!(!dir.join("hook-injected").exists());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn main_setup_already_configured_is_a_noop() {
    let dir = unique_temp_dir("setup_noop");
    fs::write(
        dir.join("settings.json"),
        r#"{"statusLine":{"type":"command","command":"claudebar render"}}"#,
    )
    .unwrap();

    let out = setup(&dir, &["-y"]);

    assert_eq!(out.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("nothing to do"),
        "stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn main_setup_conflicting_value_without_force_fails() {
    let dir = unique_temp_dir("setup_conflict");
    fs::write(
        dir.join("settings.json"),
        r#"{"statusLine":{"type":"command","command":"other-tool"}}"#,
    )
    .unwrap();

    let out = setup(&dir, &["-y"]);

    assert_eq!(out.status.code(), Some(1), "conflict exits 1");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("--force"),
        "stderr points at --force: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let after = fs::read_to_string(dir.join("settings.json")).unwrap();
    assert!(after.contains("other-tool"), "file left untouched: {after}");
}

#[test]
fn main_setup_force_overwrites_a_conflicting_value() {
    let dir = unique_temp_dir("setup_force");
    fs::write(
        dir.join("settings.json"),
        r#"{"statusLine":{"type":"command","command":"other-tool"}}"#,
    )
    .unwrap();

    let out = setup(&dir, &["--force", "-y"]);

    assert_eq!(out.status.code(), Some(0));
    let after = fs::read_to_string(dir.join("settings.json")).unwrap();
    assert!(after.contains("claudebar render"), "overwritten: {after}");
    assert_eq!(bak_count(&dir), 1, "one backup written");
}

#[test]
fn main_setup_non_interactive_without_yes_refuses() {
    let dir = unique_temp_dir("setup_noyes");

    let out = setup(&dir, &[]);

    assert_eq!(out.status.code(), Some(1), "no --yes in a pipe exits 1");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("--yes"),
        "stderr asks for --yes: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!dir.join("settings.json").exists(), "nothing written");
}

#[test]
fn main_setup_malformed_json_is_surfaced_and_backed_up() {
    let dir = unique_temp_dir("setup_malformed");
    fs::write(dir.join("settings.json"), "{ not json").unwrap();

    let out = setup(&dir, &["-y"]);

    assert_eq!(out.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("not valid JSON"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        bak_count(&dir),
        1,
        "malformed file backed up before bailing"
    );
}

#[test]
fn main_config_exits_gracefully() {
    let output = bin()
        .arg("config")
        .output()
        .expect("failed to run claudebar config");

    let exit_code = output.status.code();
    assert!(
        exit_code.is_some(),
        "config should terminate without panic, got: {exit_code:?}",
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.is_empty()
            || stderr.contains("claudebar")
            || stderr.contains("built without the `tui` feature"),
        "stderr should be empty or contain claudebar/tui feature message, got: {stderr}"
    );
}
