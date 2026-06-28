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
        #[arg(long)]
        segments: bool,
    },
    /// Sync the config file: add any new segments introduced in newer claudebar versions.
    Sync,
    /// Render a built-in fixture to verify the install works.
    Smoke,
    /// Run diagnostics: font, git, config, PATH.
    Doctor,
    /// Open the config file in $EDITOR (falls back to vi).
    Edit,
    /// Generate shell completions.
    Completions {
        /// Target shell.
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}
