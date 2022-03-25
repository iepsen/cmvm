use clap::{Parser, Subcommand};

mod cache;
mod commands;
mod constants;
mod http;
mod package;
mod platform;
mod releases;
mod versions;
mod config;

use commands::Commands;
use crate::config::Config;

#[derive(Parser)]
#[clap(version, about = "cmake version manager")]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: CliCommands,
}

#[derive(Subcommand)]
enum CliCommands {
    /// Install a cmake version
    Install { version: String },

    /// Uninstall a cmake version
    Uninstall { version: String },

    /// Use a cmake version
    Use { version: String },

    /// List all cmake versions installed
    List,

    /// List available cmake versions to install
    ListRemote,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cache::bootstrap()?;

    match Cli::parse().command {
        CliCommands::Install { version } => {
            Commands::install_version(&version)?;
        }
        CliCommands::Uninstall { version } => {
            Commands::uninstall_version(&version)?;
        }
        CliCommands::Use { version } => {
            Commands::use_version(&version)?;
        }
        CliCommands::List => {
            let config = Config::new();
            config.print_dirs()?;
            Commands::list_versions()?;
        }
        CliCommands::ListRemote => {
            Commands::list_remote_versions()?;
        }
    }
    Ok(())
}
