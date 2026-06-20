mod cli;

use clap::Parser;
use claudebar::model::Config;
use claudebar::{render_line, styles, themes, InputData};
use cli::{Cli, Command};
use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.cmd.as_ref().unwrap_or(&Command::Render) {
        Command::Render => run_render(&cli),
        Command::Config => run_config(&cli),
        Command::Init { force, print } => run_init(&cli, *force, *print),
        Command::List => run_list(),
        Command::Migrate => run_migrate(&cli),
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
        cfg.theme = t.clone();
    }
    if let Some(s) = &cli.style {
        cfg.style = s.clone();
    }
    cfg
}

fn run_render(cli: &Cli) -> ExitCode {
    let mut buf = String::new();
    // Reading stdin can't meaningfully fail the line; ignore errors and parse
    // whatever we got (InputData::parse is itself infallible).
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
    let cfg = Config::default();
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
                eprintln!("claudebar: could not determine a config path (set $HOME or --config).");
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
                eprintln!("claudebar: wrote default config to {}", path.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("claudebar: {e}");
                ExitCode::FAILURE
            }
        }
    }
}

fn run_list() -> ExitCode {
    println!("Themes:");
    for n in themes::NAMES {
        println!("  {n}");
    }
    println!("Styles:");
    for n in styles::NAMES {
        println!("  {n}");
    }
    ExitCode::SUCCESS
}

fn run_migrate(cli: &Cli) -> ExitCode {
    use claudebar::model::SegmentKind;

    let path: PathBuf = match cli.config.clone().or_else(Config::default_path) {
        Some(p) => p,
        None => {
            eprintln!("claudebar: could not determine a config path (set $HOME or --config).");
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
            "claudebar: no config file at {} — nothing to migrate.",
            path.display()
        );
        eprintln!("Run `claudebar init` to create one.");
        return ExitCode::SUCCESS;
    }

    // Find segments present in ALL but absent from the user's segments list.
    // Insert each at its canonical position relative to its neighbors in ALL.
    let mut added: Vec<&str> = Vec::new();
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
