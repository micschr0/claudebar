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
    }
}

/// Resolve the config, applying any `--theme` / `--style` overrides.
fn resolve_config(cli: &Cli) -> Config {
    let mut cfg = Config::load_or_default(cli.config.as_deref());
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
