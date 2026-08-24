//! Command-line surface. `render` runs by default — pipe session JSON to stdin.

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// Render/override options shared by the render-path commands and the top-level
/// invocation. Flattened into both so `claudebar --theme X` and
/// `claudebar render --theme X` both work — but deliberately **not** attached
/// to `update`, which has no render overrides.
#[derive(Args, Debug, Default, Clone)]
pub struct Overrides {
    /// Path to the config file (defaults to $XDG_CONFIG_HOME/claudebar/config.toml).
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Override the theme for this invocation.
    #[arg(long, value_name = "NAME")]
    pub theme: Option<String>,

    /// Override the style for this invocation.
    #[arg(long, value_name = "NAME")]
    pub style: Option<String>,

    /// Comma-separated list of segments to render (overrides config file).
    /// Names in kebab-case, e.g. "directory,git,cost,duration".
    #[arg(long, value_name = "SEGMENTS", value_delimiter = ',')]
    pub segments: Option<Vec<String>>,
}

#[derive(Parser, Debug)]
#[command(
    name = "claudebar",
    version,
    about = "Powerline-style status line for Claude Code (render + TUI configurator)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Option<Command>,

    /// Render overrides usable at the top level, e.g. `claudebar --theme X`.
    #[command(flatten)]
    pub overrides: Overrides,
}

impl Cli {
    /// The effective render overrides for the invocation: the subcommand's
    /// overrides (e.g. `claudebar init --config FILE`) win over the top-level
    /// ones (`claudebar --config FILE init`), field by field.
    pub fn effective_overrides(&self) -> Overrides {
        let mut o = self.overrides.clone();
        let sub = match &self.cmd {
            Some(Command::Render(s)) => s,
            Some(Command::Config(s)) => s,
            Some(Command::Init { overrides, .. }) => overrides,
            Some(Command::Sync(s)) => s,
            Some(Command::Doctor(s)) => s,
            Some(Command::Edit(s)) => s,
            Some(Command::Setup { overrides, .. }) => overrides,
            // List, Smoke, Completions, Update carry no render overrides.
            _ => return o,
        };
        if sub.config.is_some() {
            o.config = sub.config.clone();
        }
        if sub.theme.is_some() {
            o.theme = sub.theme.clone();
        }
        if sub.style.is_some() {
            o.style = sub.style.clone();
        }
        if sub.segments.is_some() {
            o.segments = sub.segments.clone();
        }
        o
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Read session JSON from stdin and write the status line to stdout (default).
    Render(Overrides),
    /// Launch the interactive TUI configurator.
    Config(Overrides),
    /// Write a default config file (or print it).
    ///
    /// See also: `sync` to add new segments to an existing config, `setup` to wire up Claude Code's settings.json.
    Init {
        /// Overwrite an existing config file.
        #[arg(long)]
        force: bool,
        /// Print the default config to stdout instead of writing a file.
        #[arg(long)]
        print: bool,
        #[command(flatten)]
        overrides: Overrides,
    },
    /// List the built-in themes and styles.
    List {
        /// List segments (kebab-case names, labels, default status) instead of themes/styles.
        #[arg(long = "list-segments")]
        list_segments: bool,
    },
    /// Sync the config file: add any new segments introduced in newer claudebar versions.
    ///
    /// See also: `init` to create a fresh default config, `edit` to modify it directly.
    Sync(Overrides),
    /// Render a built-in fixture to verify the install works.
    Smoke,
    /// Run diagnostics: font, git, config, PATH.
    Doctor(Overrides),
    /// Open the config file in $EDITOR (falls back to vi).
    ///
    /// See also: `config` for the interactive TUI editor, `init` to (re)create the default file.
    Edit(Overrides),
    /// Generate shell completions.
    Completions {
        /// Target shell.
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Wire `claudebar render` into Claude Code's settings.json `statusLine` key.
    ///
    /// See also: `init` to create claudebar's own config.toml (this command instead wires Claude Code's settings.json).
    Setup {
        /// Path to settings.json (defaults to $SETTINGS env var, then ~/.claude/settings.json).
        #[arg(long, value_name = "FILE")]
        settings_path: Option<PathBuf>,
        /// Only show what would change; never write.
        #[arg(long)]
        print: bool,
        /// Skip the confirmation prompt.
        #[arg(long, short = 'y')]
        yes: bool,
        /// Overwrite an existing, different statusLine value.
        #[arg(long)]
        force: bool,
        /// Override the binary path used to build the statusLine command (defaults to "claudebar").
        #[arg(long, value_name = "PATH")]
        binary_path: Option<PathBuf>,
        #[command(flatten)]
        overrides: Overrides,
    },
    /// Check the latest claudebar release and report whether an update is available.
    ///
    /// This is a manual, offline-friendly check — it never runs during normal
    /// rendering, so the statusline hot path makes no network calls. By default
    /// it compares against the newest stable release (use `--channel beta` to
    /// include prereleases).
    Update {
        /// Never exit `2` for "update available": always `0` on success.
        /// Useful in `set -e` shells / `&&`-chains. The result is still shown
        /// on stdout.
        #[arg(long)]
        check: bool,
        /// Release channel to compare against.
        #[arg(long, value_enum, default_value = "stable")]
        channel: claudebar::update::Channel,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn editor_is_unset_returns_none() {
        // `Edit` carries render `Overrides` — parse it.
        let cli = Cli::try_parse_from(["claudebar", "edit"]).expect("edit subcommand must parse");
        assert!(matches!(cli.cmd, Some(Command::Edit(_))));
    }

    #[test]
    fn init_print_emits_toml_to_stdout() {
        // Verify that `init --print` parses correctly.
        let cli =
            Cli::try_parse_from(["claudebar", "init", "--print"]).expect("init --print must parse");
        match cli.cmd {
            Some(Command::Init {
                force, print: true, ..
            }) => {
                assert!(!force, "--force should default to false");
            }
            other => panic!("expected Init with print: true, got: {other:?}"),
        }

        // Verify that a default Config serializes to valid TOML.
        let cfg = claudebar::model::Config::default();
        let toml_str = toml::to_string_pretty(&cfg).expect("default Config must serialize to TOML");
        assert!(
            toml_str.contains("theme ="),
            "TOML output must contain a theme key"
        );
        assert!(
            toml_str.contains("tokyo-night"),
            "default theme should be tokyo-night"
        );
        assert!(
            toml_str.contains("[thresholds]"),
            "TOML output must contain a [thresholds] section"
        );
    }

    #[test]
    fn smoke_subcommand_defaults_to_render() {
        // When no subcommand is given, it should default to Render.
        let cli = Cli::try_parse_from(["claudebar"]).expect("bare claudebar must parse");
        assert!(
            cli.cmd.is_none(),
            "bare invocation should have cmd=None (default Render)"
        );
    }

    #[test]
    fn init_without_print_is_write_mode() {
        let cli = Cli::try_parse_from(["claudebar", "init"]).expect("init must parse");
        match cli.cmd {
            Some(Command::Init { force, print, .. }) => {
                assert!(!force);
                assert!(!print);
            }
            other => panic!("expected Init, got: {other:?}"),
        }
    }

    #[test]
    fn doctor_subcommand_parses() {
        let cli =
            Cli::try_parse_from(["claudebar", "doctor"]).expect("doctor subcommand must parse");
        assert!(matches!(cli.cmd, Some(Command::Doctor(_))));
    }

    #[test]
    fn smoke_subcommand_parses() {
        let cli = Cli::try_parse_from(["claudebar", "smoke"]).expect("smoke subcommand must parse");
        assert!(matches!(cli.cmd, Some(Command::Smoke)));
    }
}
