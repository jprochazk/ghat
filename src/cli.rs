pub mod add;
pub mod common;
pub mod generate;
pub mod init;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ghat", about = "GitHub Actions Templating system and runtime")]
struct Cli {
    /// Enable verbose output (-v for debug, -vv for trace)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// GitHub API token
    #[arg(long, global = true, env = "GITHUB_TOKEN")]
    github_token: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize project structure (.github/ghat/)
    Init,

    /// Add actions to the lockfile (pinned to commit sha)
    Add {
        /// Actions to add (e.g. Swatinem/rust-cache taiki-e/install-action)
        #[arg(conflicts_with = "auto")]
        actions: Vec<String>,

        /// Scan workflow definitions and add all referenced actions automatically
        #[arg(long, conflicts_with = "actions")]
        auto: bool,
    },

    /// Remove actions from the lockfile
    ///
    /// Without arguments, displays an interactive list showing each action
    /// and whether it is currently used in any workflow definition.
    Rm {
        /// Actions to remove (interactive selection if omitted)
        actions: Vec<String>,
    },

    /// Update actions to their latest version
    Update {
        /// Actions to update (all if omitted)
        actions: Vec<String>,
    },

    /// Generate workflow files from definitions
    Generate,

    /// Check workflow definitions without writing files
    Check,
}

pub fn entrypoint() -> miette::Result<()> {
    let cli = Cli::parse();

    let log_level = match cli.verbose {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    env_logger::Builder::new()
        .filter_level(log_level)
        .format_target(false)
        .format_timestamp(None)
        .init();

    match cli.command {
        Command::Init => init::run(),
        Command::Add { actions, auto } => add::run(actions, auto, cli.github_token),
        Command::Rm { actions } => todo!("rm: {actions:?}"),
        Command::Update { actions } => todo!("update: {actions:?}"),
        Command::Generate => generate::run(),
        Command::Check => todo!("check"),
    }
}
