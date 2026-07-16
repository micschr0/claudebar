//! Integration smoke test: spawn the `claudebar` binary with fixture data
//! and verify exit-0 / non-empty output.

use std::io::{Read, Write};
use std::process::Command;

#[test]
fn smoke_returns_zero_exit_code() {
    let fixture = br#"{
        "cwd": "/home/user/project",
        "context_window": {
            "total_input_tokens": 35000,
            "total_output_tokens": 7300,
            "used_percentage": 67.0
        },
        "rate_limits": {
            "five_hour": { "used_percentage": 48.0, "resets_at": 1900000000 }
        },
        "model": { "display_name": "Opus 4.8" }
    }"#;

    let mut child = Command::new(env!("CARGO_BIN_EXE_claudebar"))
        .arg("render")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn claudebar render");

    // Write fixture data to stdin and close.
    let mut stdin = child.stdin.take().expect("failed to open stdin");
    stdin.write_all(fixture).expect("failed to write fixture");
    drop(stdin);

    let mut stdout = String::new();
    child
        .stdout
        .take()
        .expect("failed to open stdout")
        .read_to_string(&mut stdout)
        .expect("failed to read stdout");
    let status = child.wait().expect("failed to wait on child");

    assert!(status.success(), "exit code should be 0, got: {status:?}");
    assert!(!stdout.is_empty(), "stdout should have output, got empty");
}

#[test]
fn smoke_with_smoke_subcommand_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_claudebar"))
        .arg("smoke")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .expect("failed to run claudebar smoke");

    assert!(output.status.success(), "smoke subcommand should exit 0");
    assert!(
        !output.stdout.is_empty(),
        "smoke output should not be empty"
    );
}
