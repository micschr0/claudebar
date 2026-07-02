mod cli;

use clap::Parser;
use claudebar::model::{Config, SegmentKind};
use claudebar::{InputData, render_line, styles, themes};
use cli::{Cli, Command};
use std::io::{IsTerminal, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.cmd.as_ref().unwrap_or(&Command::Render) {
        Command::Render => run_render(&cli),
        Command::Config => run_config(&cli),
        Command::Init { force, print } => run_init(&cli, *force, *print),
        Command::List { list_segments } => run_list(*list_segments),
        Command::Sync => run_sync(&cli),
        Command::Smoke => run_smoke(),
        Command::Doctor => run_doctor(&cli),
        Command::Edit => run_edit(&cli),
        Command::Completions { shell } => run_completions(*shell),
        Command::Setup {
            settings_path,
            print,
            yes,
            force,
        } => run_setup(&cli, settings_path.clone(), *print, *yes, *force),
    }
}

/// Resolve the config, applying any `--theme` / `--style` overrides.
/// Warns on stderr when the config file exists but cannot be parsed.
fn resolve_config(cli: &Cli) -> Config {
    let path = cli.config.clone().or_else(Config::default_path);
    let mut cfg = match path {
        Some(ref p) => {
            if p.exists() {
                match Config::load(p) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("claudebar: warning: {e} — using defaults");
                        Config::default()
                    }
                }
            } else {
                Config::load_or_default(cli.config.as_deref())
            }
        }
        None => Config::default(),
    };
    if let Some(t) = &cli.theme {
        if !themes::NAMES.contains(&t.as_str()) {
            eprintln!("claudebar: warning: unknown theme '{t}' — using tokyo-night");
        }
        cfg.theme = t.clone();
    }
    if let Some(s) = &cli.style {
        if !styles::NAMES.contains(&s.as_str()) {
            eprintln!("claudebar: warning: unknown style '{s}' — using powerline");
        }
        cfg.style = s.clone();
    }
    if let Some(segs) = &cli.segments {
        let mut parsed: Vec<SegmentKind> = Vec::with_capacity(segs.len());
        for s in segs {
            match SegmentKind::from_kebab(s) {
                Some(k) => parsed.push(k),
                None => eprintln!("claudebar: warning: unknown segment '{s}' — ignored"),
            }
        }
        if !parsed.is_empty() {
            cfg.segments = parsed;
        }
    }
    cfg
}
fn run_render(cli: &Cli) -> ExitCode {
    let mut buf = String::new();
    if std::io::stdin().is_terminal() {
        eprintln!("claudebar: reading from stdin — pipe session JSON or press Ctrl+D");
    }
    let _ = std::io::stdin().read_to_string(&mut buf);
    let input = InputData::parse(&buf);
    let cfg = resolve_config(cli);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    println!("{}", render_line(&input, &cfg, now));
    ExitCode::SUCCESS
}

fn run_config(cli: &Cli) -> ExitCode {
    #[cfg(feature = "tui")]
    {
        let path = cli.config.clone().or_else(Config::default_path);
        match claudebar::tui::run(path) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("claudebar: {e}");
                ExitCode::FAILURE
            }
        }
    }
    #[cfg(not(feature = "tui"))]
    {
        let _ = cli;
        eprintln!("claudebar: built without the `tui` feature — configurator unavailable.");
        eprintln!("Edit the TOML config directly (see `claudebar init --print`).");
        ExitCode::FAILURE
    }
}

fn run_init(cli: &Cli, force: bool, print: bool) -> ExitCode {
    let (cfg, no_nerd_font) = default_config_with_font_check();
    if print {
        match toml::to_string_pretty(&cfg) {
            Ok(s) => {
                print!("{s}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("claudebar: {e}");
                ExitCode::FAILURE
            }
        }
    } else {
        let path: PathBuf = match cli.config.clone().or_else(Config::default_path) {
            Some(p) => p,
            None => {
                eprintln!("claudebar: no config path found — set $HOME or use --config.");
                return ExitCode::FAILURE;
            }
        };
        if path.exists() && !force {
            eprintln!(
                "claudebar: {} already exists (use --force to overwrite).",
                path.display()
            );
            return ExitCode::FAILURE;
        }
        match cfg.save(&path) {
            Ok(()) => {
                println!("claudebar: wrote default config to {}", path.display());
                if no_nerd_font {
                    print_nerd_font_hint();
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("claudebar: {e}");
                ExitCode::FAILURE
            }
        }
    }
}

/// Shared by `run_init` and `run_edit` so both bootstrap paths agree on the
/// default: builds a default `Config`, falling back to style `"unicode"`
/// when no Nerd Font is detected. Returns the config plus whether the
/// fallback was applied (`true` when no Nerd Font was found).
fn default_config_with_font_check() -> (Config, bool) {
    let mut cfg = Config::default();
    let no_nerd_font = !check_nerd_font();
    if no_nerd_font {
        cfg.style = "unicode".into();
    }
    (cfg, no_nerd_font)
}

/// Print the "no Nerd Font detected" hint block shared by `run_init` and `run_edit`.
fn print_nerd_font_hint() {
    println!("claudebar: no Nerd Font detected — falling back to style \"unicode\" in config.");
    println!(
        "claudebar: install a Nerd Font (https://www.nerdfonts.com) and run `claudebar config` to switch to `powerline`."
    );
    if cfg!(target_os = "macos") {
        println!("claudebar:   macOS: brew install --cask font-hack-nerd-font");
    }
}

/// Print the `statusLine:` diff block shared by the `WillSet` and `Conflict` outcomes.
fn print_status_line_diff(previous: Option<&serde_json::Value>, desired: &serde_json::Value) {
    let old_line = previous.map_or_else(
        || "(none)".to_string(),
        |v| serde_json::to_string(v).unwrap_or_else(|_| "(none)".to_string()),
    );
    let new_line = serde_json::to_string(desired).unwrap_or_default();
    println!("statusLine:");
    println!("- {old_line}");
    println!("+ {new_line}");
}

fn run_setup(
    cli: &Cli,
    settings_path: Option<PathBuf>,
    print: bool,
    yes: bool,
    force: bool,
) -> ExitCode {
    let path = match settings_path.or_else(claudebar::setup::default_settings_path) {
        Some(p) => p,
        None => {
            eprintln!(
                "claudebar: no settings path found — set $HOME, $SETTINGS, or use --settings-path."
            );
            return ExitCode::FAILURE;
        }
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let existed = path.exists();

    let mut settings = match claudebar::setup::load_settings(&path) {
        Ok(s) => s,
        Err(claudebar::setup::SetupError::Parse(msg)) => {
            if existed {
                match claudebar::setup::backup_settings(&path, now) {
                    Ok(backup_path) => {
                        println!(
                            "claudebar: backed up existing settings to {}",
                            backup_path.display()
                        );
                    }
                    Err(e) => eprintln!("claudebar: {e}"),
                }
            }
            eprintln!("claudebar: {} is not valid JSON: {msg}", path.display());
            return ExitCode::FAILURE;
        }
        Err(claudebar::setup::SetupError::Io(msg)) => {
            eprintln!("claudebar: {msg}");
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("claudebar: {e}");
            return ExitCode::FAILURE;
        }
    };

    let desired = claudebar::setup::desired_status_line();

    match claudebar::setup::classify(&settings, &desired, force) {
        claudebar::setup::Outcome::AlreadyConfigured => {
            println!("claudebar: statusLine already configured — nothing to do.");
            print_setup_preview(cli);
            ExitCode::SUCCESS
        }
        claudebar::setup::Outcome::Conflict { existing } => {
            print_status_line_diff(Some(&existing), &desired);
            eprintln!(
                "claudebar: statusLine is set to a different value — rerun with --force to overwrite."
            );
            ExitCode::FAILURE
        }
        claudebar::setup::Outcome::WillSet { previous } => {
            print_status_line_diff(previous.as_ref(), &desired);

            if print {
                return ExitCode::SUCCESS;
            }

            let confirmed = if yes {
                true
            } else if std::io::stdin().is_terminal() {
                print!("Apply this change? [y/N] ");
                let _ = std::io::stdout().flush();
                let mut answer = String::new();
                let _ = std::io::stdin().read_line(&mut answer);
                let answer = answer.trim().to_lowercase();
                answer == "y" || answer == "yes"
            } else {
                eprintln!("claudebar: non-interactive session — pass --yes to confirm.");
                return ExitCode::FAILURE;
            };

            if !confirmed {
                println!("claudebar: aborted.");
                return ExitCode::FAILURE;
            }

            if existed {
                match claudebar::setup::backup_settings(&path, now) {
                    Ok(backup_path) => {
                        println!(
                            "claudebar: backed up existing settings to {}",
                            backup_path.display()
                        );
                    }
                    Err(e) => {
                        eprintln!("claudebar: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            }

            claudebar::setup::apply(&mut settings, desired);
            match claudebar::setup::save_settings(&path, &settings) {
                Ok(()) => {
                    println!("claudebar: statusLine configured -> {}", path.display());
                    print_setup_preview(cli);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("claudebar: {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

/// Mirrors `run_smoke`'s fixture/now setup, but resolves the user's real
/// config via `resolve_config` instead of always rendering the default.
/// Exists so `setup` proves the wiring works without the user needing to
/// restart Claude Code and hope.
fn print_setup_preview(cli: &Cli) {
    let fixture = include_str!("../fixtures/typical.json");
    let input = InputData::parse(fixture);
    let cfg = resolve_config(cli);
    let now: i64 = 1_900_000_000;
    let output = render_line(&input, &cfg, now);
    println!();
    println!("Preview:");
    println!("{output}");
}

fn run_list(segments: bool) -> ExitCode {
    if segments {
        println!("Segments:");
        for &kind in &SegmentKind::ALL {
            let kebab = serde_json::to_string(&kind)
                .unwrap_or_else(|_| String::from("?"))
                .trim_matches('"')
                .to_string();
            let in_default = SegmentKind::DEFAULT.contains(&kind);
            let default_mark = if in_default { "  [default]" } else { "" };
            println!("  {kebab}  —  {}{default_mark}", kind.label());
        }
    } else {
        println!("Themes:");
        for n in themes::NAMES {
            println!("  {n}");
        }
        println!("Styles:");
        for n in styles::NAMES {
            println!("  {n}");
        }
    }
    ExitCode::SUCCESS
}

fn run_sync(cli: &Cli) -> ExitCode {
    use claudebar::model::SegmentKind;

    let path: PathBuf = match cli.config.clone().or_else(Config::default_path) {
        Some(p) => p,
        None => {
            eprintln!("claudebar: no config path found — set $HOME or use --config.");
            return ExitCode::FAILURE;
        }
    };

    // Load the existing config. If the file doesn't exist, there's nothing to migrate
    // (the default already includes all segments).
    let mut cfg = match Config::load(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("claudebar: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Check if the config file actually exists (load returns default for missing files).
    if !path.exists() {
        eprintln!(
            "claudebar: no config file at {} — nothing to sync.",
            path.display()
        );
        eprintln!("Run `claudebar init` to create one.");
        return ExitCode::SUCCESS;
    }

    // Find segments present in ALL but absent from the user's segments list.
    // Insert each at its canonical position relative to its neighbors in ALL.
    let mut added: Vec<&str> = Vec::with_capacity(SegmentKind::ALL.len());
    for (canonical_pos, &kind) in SegmentKind::ALL.iter().enumerate() {
        if cfg.segments.contains(&kind) {
            continue;
        }
        // Find the best insertion point: after the last ALL-segment that is already
        // in cfg.segments and comes before `kind` in canonical order.
        let insert_after = SegmentKind::ALL[..canonical_pos]
            .iter()
            .rev()
            .find_map(|&predecessor| cfg.segments.iter().rposition(|s| *s == predecessor));

        let pos = match insert_after {
            Some(idx) => idx + 1,
            // No known predecessor present — insert before the first ALL-segment
            // that is already in cfg.segments and comes after `kind`.
            None => SegmentKind::ALL[canonical_pos + 1..]
                .iter()
                .find_map(|&successor| cfg.segments.iter().position(|s| *s == successor))
                .unwrap_or(cfg.segments.len()),
        };

        cfg.segments.insert(pos, kind);
        added.push(kind.label());
    }

    if added.is_empty() {
        println!("claudebar: config is up to date, no segments added.");
        return ExitCode::SUCCESS;
    }

    match cfg.save(&path) {
        Ok(()) => {
            for label in &added {
                println!("claudebar: added segment '{label}'");
            }
            println!("claudebar: config updated: {}", path.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("claudebar: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run_smoke() -> ExitCode {
    let fixture = include_str!("../fixtures/typical.json");
    let input = InputData::parse(fixture);
    let cfg = Config::default();
    // Use a fixed "now" for deterministic output; HOME is whatever the env provides.
    let now: i64 = 1_900_000_000;
    let output = render_line(&input, &cfg, now);
    println!("{output}");
    println!();
    println!("Looks right? Run `claudebar doctor` to check your environment.");
    ExitCode::SUCCESS
}

fn run_completions(shell: clap_complete::Shell) -> ExitCode {
    let mut cmd = <Cli as clap::CommandFactory>::command();
    let name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, &name, &mut std::io::stdout());
    ExitCode::SUCCESS
}

fn run_doctor(cli: &Cli) -> ExitCode {
    // 1. Check: binary in PATH?
    let in_path = match std::env::current_exe() {
        Ok(exe) => {
            if let Some(dir) = exe.parent() {
                std::env::var_os("PATH")
                    .map(|p| std::env::split_paths(&p).any(|d| d == dir))
                    .unwrap_or(false)
            } else {
                false
            }
        }
        Err(_) => false,
    };
    println!("{} Binary in PATH", check_mark(in_path));

    // 2. Nerd Font installed?
    let has_nerd = check_nerd_font();
    println!("{} Nerd Font installed", check_mark(has_nerd));

    // 3. git on PATH?
    let has_git = which_ok("git");
    println!("{} git on PATH", check_mark(has_git));

    // 4. config.toml parses?
    let config_path = cli.config.clone().or_else(Config::default_path);
    let config_ok = match config_path {
        Some(ref p) if p.exists() => Config::load(p).is_ok(),
        _ => true, // no config or default path = fine
    };
    println!("{} config.toml parses", check_mark(config_ok));

    // 5. statusLine configured in Claude Code's settings.json?
    let status_line_ok = claudebar::setup::default_settings_path()
        .and_then(|p| claudebar::setup::load_settings(&p).ok())
        .and_then(|settings| {
            settings
                .get("statusLine")
                .and_then(|sl| sl.get("command"))
                .and_then(|c| c.as_str())
                .map(|s| s.contains("claudebar"))
        })
        .unwrap_or(false);
    println!(
        "{} statusLine configured (run `claudebar setup` if not)",
        check_mark(status_line_ok)
    );

    println!();
    println!("Rendering look off? Run `claudebar smoke` to test with fixture data.");

    // Always succeed — this is informational.
    ExitCode::SUCCESS
}

fn run_edit(cli: &Cli) -> ExitCode {
    let path: PathBuf = match cli.config.clone().or_else(Config::default_path) {
        Some(p) => p,
        None => {
            eprintln!("claudebar: no config path found — set $HOME or use --config.");
            return ExitCode::FAILURE;
        }
    };

    // Init if missing.
    if !path.exists() {
        let (cfg, no_nerd_font) = default_config_with_font_check();
        if let Err(e) = cfg.save(&path) {
            eprintln!("claudebar: failed to create config: {e}");
            return ExitCode::FAILURE;
        }
        eprintln!("claudebar: created default config at {}", path.display());
        if no_nerd_font {
            print_nerd_font_hint();
        }
    }

    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| String::from("vi"));

    let status = std::process::Command::new(&editor).arg(&path).status();

    match status {
        Ok(s) if s.success() => ExitCode::SUCCESS,
        Ok(s) => {
            eprintln!("claudebar: {editor} exited with status {}", s);
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("claudebar: failed to launch {editor}: {e}");
            ExitCode::FAILURE
        }
    }
}

fn check_mark(ok: bool) -> &'static str {
    if ok { "✓" } else { "✗" }
}

/// Nerd Font check: look for .ttf/.otf files with "Nerd" or "nerd" in their name.
/// Uses `fc-list` if available; falls back to scanning common font dirs.
fn check_nerd_font() -> bool {
    // Try fc-list first — fastest and most accurate.
    if let Ok(output) = std::process::Command::new("fc-list")
        .arg(":family")
        .output()
        && let Ok(stdout) = String::from_utf8(output.stdout)
        && stdout.to_lowercase().contains("nerd")
    {
        return true;
    }

    // Fallback: scan common font directories.
    let dirs: &[&str] = &[
        "/usr/share/fonts",
        "/usr/local/share/fonts",
        "~/.local/share/fonts",
        "~/.fonts",
    ];

    let home = std::env::var("HOME").unwrap_or_default();

    for dir in dirs {
        let path = if let Some(stripped) = dir.strip_prefix("~/") {
            PathBuf::from(home.clone()).join(stripped)
        } else {
            PathBuf::from(dir)
        };
        if path.is_dir()
            && let Ok(entries) = std::fs::read_dir(&path)
        {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if (name.ends_with(".ttf") || name.ends_with(".otf"))
                    && name.to_lowercase().contains("nerd")
                {
                    return true;
                }
            }
        }
    }

    false
}

fn which_ok(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_with_font_check_style_matches_flag() {
        let (cfg, no_nerd_font) = default_config_with_font_check();
        if no_nerd_font {
            assert_eq!(cfg.style, "unicode");
        } else {
            assert_eq!(cfg.style, Config::default().style);
        }
    }
}
