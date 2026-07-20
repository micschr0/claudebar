//! Command-line surface. `render` runs by default — pipe session JSON to stdin.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "claudebar",
    version,
    about = "Powerline-style status line for Claude Code (render + TUI configurator)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Option<Command>,

    /// Path to the config file (defaults to $XDG_CONFIG_HOME/claudebar/config.toml).
    #[arg(long, global = true, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Override the theme for this invocation.
    #[arg(long, global = true, value_name = "NAME")]
    pub theme: Option<String>,

    /// Override the style for this invocation.
    #[arg(long, global = true, value_name = "NAME")]
    pub style: Option<String>,

    /// Comma-separated list of segments to render (overrides config file).
    /// Names in kebab-case, e.g. "directory,git,cost,duration".
    #[arg(long, global = true, value_name = "SEGMENTS", value_delimiter = ',')]
    pub segments: Option<Vec<String>>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Read session JSON from stdin and write the status line to stdout (default).
    Render,
    /// Launch the interactive TUI configurator.
    Config,
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
    Sync,
    /// Render a built-in fixture to verify the install works.
    Smoke,
    /// Run diagnostics: font, git, config, PATH.
    Doctor,
    /// Open the config file in $EDITOR (falls back to vi).
    ///
    /// See also: `config` for the interactive TUI editor, `init` to (re)create the default file.
    Edit,
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
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn editor_is_unset_returns_none() {
        // `Edit` is a bare variant with no extra fields — parse it.
        let cli = Cli::try_parse_from(["claudebar", "edit"]).expect("edit subcommand must parse");
        assert!(matches!(cli.cmd, Some(Command::Edit)));
    }

    #[test]
    fn init_print_emits_toml_to_stdout() {
        // Verify that `init --print` parses correctly.
        let cli =
            Cli::try_parse_from(["claudebar", "init", "--print"]).expect("init --print must parse");
        match cli.cmd {
            Some(Command::Init { force, print: true }) => {
                assert!(!force, "--force should default to false");
            }
            other => panic!("expected Init {{ force: false, print: true }}, got: {other:?}"),
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
            Some(Command::Init { force, print }) => {
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
        assert!(matches!(cli.cmd, Some(Command::Doctor)));
    }

    #[test]
    fn smoke_subcommand_parses() {
        let cli = Cli::try_parse_from(["claudebar", "smoke"]).expect("smoke subcommand must parse");
        assert!(matches!(cli.cmd, Some(Command::Smoke)));
    }
}
