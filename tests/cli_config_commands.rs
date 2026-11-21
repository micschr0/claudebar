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
fn setup_creates_settings_with_private_mode_respecting_umask() {
    for (mask, expected) in [("000", 0o600), ("200", 0o400)] {
        let dir = temp_dir(mask);
        let settings = dir.join("settings.json");
        let output = Command::new("sh")
            .arg("-c")
            .arg("umask \"$1\"; exec \"$2\" setup --config \"$3\" --settings-path \"$4\" -y")
            .arg("sh")
            .arg(mask)
            .arg(env!("CARGO_BIN_EXE_claudebar"))
            .arg(dir.join("config.toml"))
            .arg(&settings)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            fs::metadata(&settings).unwrap().permissions().mode() & 0o777,
            expected
        );
        fs::remove_dir_all(dir).unwrap();
    }
}

#[cfg(target_os = "linux")]
mod acl_tests {
    use super::*;
    use std::ffi::{CStr, CString, c_char, c_int, c_void};
    use std::os::unix::{ffi::OsStrExt, fs::MetadataExt};

    unsafe extern "C" {
        fn setxattr(
            path: *const c_char,
            name: *const c_char,
            value: *const c_void,
            size: usize,
            flags: c_int,
        ) -> c_int;
        fn getxattr(
            path: *const c_char,
            name: *const c_char,
            value: *mut c_void,
            size: usize,
        ) -> isize;
    }

    fn acl(uid: u32, user: u16, mask: u16) -> Vec<u8> {
        let mut bytes = 2_u32.to_le_bytes().to_vec();
        for (tag, permissions, id) in [
            (1_u16, user, u32::MAX),
            (2, 4, uid),
            (4, 4, u32::MAX),
            (16, mask, u32::MAX),
            (32, 0, u32::MAX),
        ] {
            bytes.extend(tag.to_le_bytes());
            bytes.extend(permissions.to_le_bytes());
            bytes.extend(id.to_le_bytes());
        }
        bytes
    }

    fn set_acl(path: &Path, name: &CStr, value: &[u8]) -> std::io::Result<()> {
        let path = CString::new(path.as_os_str().as_bytes()).unwrap();
        // SAFETY: All pointers are valid for the duration of the call.
        let result = unsafe {
            setxattr(
                path.as_ptr(),
                name.as_ptr(),
                value.as_ptr().cast(),
                value.len(),
                0,
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    fn read_acl(path: &Path) -> Vec<u8> {
        let path = CString::new(path.as_os_str().as_bytes()).unwrap();
        let mut value = vec![0; 44];
        // SAFETY: The buffer is writable and its length is passed to getxattr.
        let size = unsafe {
            getxattr(
                path.as_ptr(),
                c"system.posix_acl_access".as_ptr(),
                value.as_mut_ptr().cast(),
                value.len(),
            )
        };
        assert_eq!(size, 44, "getxattr: {}", std::io::Error::last_os_error());
        value
    }

    #[test]
    fn setup_preserves_existing_acl_and_restricts_inherited_acl() {
        let dir = temp_dir("acl");
        let existing = dir.join("existing.json");
        fs::write(&existing, "{}").unwrap();
        let uid = fs::metadata(&existing).unwrap().uid();
        let existing_acl = acl(uid, 6, 4);
        if let Err(error) = set_acl(&existing, c"system.posix_acl_access", &existing_acl) {
            if error.kind() == std::io::ErrorKind::Unsupported {
                eprintln!("ACL test skipped: filesystem does not support POSIX ACLs: {error}");
                fs::remove_dir_all(dir).unwrap();
                return;
            }
            panic!("setting access ACL failed: {error}");
        }
        assert_eq!(read_acl(&existing), existing_acl);
        let before = fs::metadata(&existing).unwrap();
        let output = setup(&existing, &["-y"]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let after = fs::metadata(&existing).unwrap();
        assert_eq!(
            (after.dev(), after.ino(), after.mode()),
            (before.dev(), before.ino(), before.mode())
        );
        assert_eq!(read_acl(&existing), existing_acl);

        let inherited = dir.join("inherited");
        fs::create_dir(&inherited).unwrap();
        set_acl(&inherited, c"system.posix_acl_default", &acl(uid, 7, 4))
            .expect("setting default ACL on ACL-capable filesystem");
        let new_file = inherited.join("settings.json");
        let output = setup(&new_file, &["-y"]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            fs::metadata(&new_file).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(read_acl(&new_file), acl(uid, 6, 0));
        fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn setup_creates_missing_symlink_target() {
    let dir = temp_dir("dangling_settings");
    let settings = dir.join("settings.json");
    let target = dir.join("target.json");
    std::os::unix::fs::symlink("target.json", &settings).unwrap();
    let out = setup(&settings, &["-y"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(fs::read_link(&settings).unwrap(), Path::new("target.json"));
    assert_eq!(
        fs::metadata(&target).unwrap().permissions().mode() & 0o177,
        0
    );
    let written: serde_json::Value = serde_json::from_slice(&fs::read(target).unwrap()).unwrap();
    assert_eq!(written["statusLine"]["command"], "claudebar render");
    fs::remove_dir_all(dir).unwrap();
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
