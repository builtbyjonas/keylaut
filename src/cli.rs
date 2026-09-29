//! Command-line interface definitions and commands.

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "keylaut",
    author = "ByJonas",
    version,
    about = "A tiny, native keyboard utility that makes German characters effortless on English keyboard layouts.",
    long_about = "Keylaut watches for sequences like ae, oe, ue, ss and converts them into ä, ö, ü, ß\n\
                  when confirmed by context or punctuation, without changing your keyboard layout.\n\
                  Tiny. Native. Private. Instant."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Run with verbose diagnostic logging (never logs personal keystrokes)
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run the Keylaut background listener (default when no command is given)
    Run,

    /// Start Keylaut as a background service via system launcher
    Start,

    /// Stop the background Keylaut process or service
    Stop,

    /// Display Keylaut operational and autostart status
    Status,

    /// Enable Keylaut character transformation in configuration
    Enable,

    /// Disable Keylaut character transformation without terminating process
    Disable,

    /// Manage system startup behavior
    Autostart(AutostartArgs),

    /// Inspect or manage Keylaut configuration
    Config(ConfigArgs),
}

#[derive(Args, Debug)]
pub struct AutostartArgs {
    #[command(subcommand)]
    pub action: AutostartAction,
}

#[derive(Subcommand, Debug)]
pub enum AutostartAction {
    /// Enable Keylaut to automatically start at user login
    Enable,

    /// Disable Keylaut automatic startup at user login
    Disable,

    /// Check whether automatic startup is configured
    Status,
}

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: Option<ConfigAction>,
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Print the path to the configuration file
    Path,

    /// Display the active configuration contents
    Show,
}
