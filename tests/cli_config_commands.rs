//! Regressions for setup and edit shell command handling.
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "claudebar_cli_config_{name}_{}_{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_claudebar"))
}

fn setup(path: &Path, args: &[&str]) -> Output {
    bin()
        .arg("setup")
        .arg("--config")
        .arg(path.with_extension("toml"))
        .arg("--settings-path")
        .arg(path)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn setup_quoted_binary_path_executes_with_spaces_apostrophe_and_metacharacters() {
    let dir = temp_dir("binary");
    let executable = dir.join("bin 'safe' $; name");
    let marker = dir.join("called");
    fs::write(
        &executable,
        format!("#!/bin/sh\nprintf '%s' \"$1\" > '{}'\n", marker.display()),
    )
    .unwrap();
    let mut permissions = fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&executable, permissions).unwrap();
    let settings = dir.join("settings.json");
    let out = setup(
        &settings,
        &["--binary-path", executable.to_str().unwrap(), "-y"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let written: serde_json::Value = serde_json::from_slice(&fs::read(&settings).unwrap()).unwrap();
    let hook = &written["statusLine"]["command"];
    let output = Command::new("sh")
        .arg("-c")
        .arg(hook.as_str().unwrap())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "hook did not execute intended binary"
    );
    assert_eq!(fs::read_to_string(marker).unwrap(), "render");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn edit_editor_flags_and_config_path_are_safe_data() {
    let dir = temp_dir("editor");
    let config = dir.join("config file; touch pwned.toml");
    let capture = dir.join("capture");
    let script = dir.join("editor");
    fs::write(
        &script,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$1\" \"$2\" > '{}'\n",
            capture.display()
        ),
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();
    let editor = format!("{} --flag", script.display());
    let out = bin()
        .arg("edit")
        .arg("--config")
        .arg(&config)
        .env("EDITOR", editor)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        fs::read_to_string(capture).unwrap(),
        format!("--flag\n{}\n", config.display())
    );
    assert!(!dir.join("pwned.toml").exists());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn edit_propagates_editor_failure() {
    let dir = temp_dir("editor_failure");
    let config = dir.join("config.toml");
    fs::write(&config, "theme = 'tokyo-night'\n").unwrap();
    let out = bin()
        .arg("edit")
        .arg("--config")
        .arg(&config)
        .env("EDITOR", "sh -c 'exit 7'")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("exited with status"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn edit_rejects_malformed_editor_command() {
    let dir = temp_dir("editor_parse");
    let config = dir.join("config.toml");
    fs::write(&config, "theme = 'tokyo-night'\n").unwrap();
    let out = bin()
        .arg("edit")
        .arg("--config")
        .arg(&config)
        .env("EDITOR", "sh -c '")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn setup_print_malformed_json_does_not_write_and_normal_setup_still_backs_up() {
    let dir = temp_dir("malformed");
    let settings = dir.join("settings.json");
    fs::write(&settings, "{ not json").unwrap();
    let print = setup(&settings, &["--print"]);
    assert_eq!(print.status.code(), Some(1));
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
    assert_eq!(fs::read_to_string(&settings).unwrap(), "{ not json");

    let normal = setup(&settings, &["-y"]);
    assert_eq!(normal.status.code(), Some(1));
    let backups: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().contains(".bak-"))
        .collect();
    assert_eq!(backups.len(), 1);
    fs::remove_dir_all(&dir).unwrap();
}
