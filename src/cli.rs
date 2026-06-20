//! Command-line surface. `render` is the default when no subcommand is given,
//! so the hook can invoke the bare binary and pipe JSON to its stdin.

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
    List,
    /// Add any new segments (from a newer claudebar version) to an existing config.
    Migrate,
}
