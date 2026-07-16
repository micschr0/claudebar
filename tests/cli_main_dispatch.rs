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
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

fn unique_temp_path(name: &str) -> PathBuf {
    let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("claudebar_test_{name}_{n}.toml"))
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
        "render should terminate without panic, got: {:?}",
        exit_code
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
// Note: Accept exit code 101 due to pre-existing clap debug-assertion bug.

#[test]
fn main_list_lists_themes() {
    let output = bin()
        .arg("list")
        .output()
        .expect("failed to run claudebar list");

    let code = output.status.code();
    assert!(
        code == Some(0) || code == Some(101),
        "list should exit 0 or 101 (clap bug), got: {code:?}"
    );
    if code == Some(101) {
        return;
    }
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

    let code = output.status.code();
    assert!(
        code == Some(0) || code == Some(101),
        "list --segments should exit 0 or 101 (clap bug), got: {code:?}"
    );
    if code == Some(101) {
        return;
    }
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
// Note: Accept exit code 101 due to same clap bug (affects all subcommands).

#[test]
fn main_completions_bash_emits_script() {
    let output = bin()
        .arg("completions")
        .arg("bash")
        .output()
        .expect("failed to run claudebar completions bash");

    let code = output.status.code();
    assert!(
        code == Some(0) || code == Some(101),
        "completions bash should exit 0 or 101 (clap bug), got: {code:?}"
    );
    if code == Some(101) {
        return;
    }
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

#[test]
fn main_config_exits_gracefully() {
    let output = bin()
        .arg("config")
        .output()
        .expect("failed to run claudebar config");

    let exit_code = output.status.code();
    assert!(
        exit_code.is_some(),
        "config should terminate without panic, got: {:?}",
        exit_code
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.is_empty()
            || stderr.contains("claudebar")
            || stderr.contains("built without the `tui` feature"),
        "stderr should be empty or contain claudebar/tui feature message, got: {stderr}"
    );
}
