use clap::{Parser, Subcommand};

mod cache;
mod commands;
mod storage;
mod constants;
mod http;
mod package;
mod platform;
mod releases;
mod versions;

use commands::Commands;

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
    Install { v: String },

    /// Uninstall a cmake version
    Uninstall { v: String },

    /// Use a cmake version
    Use { v: String },

    /// List all cmake versions installed
    List,

    /// List available cmake versions to install
    ListRemote,

    /// Show how to put cmake current version on PATH env variable
    Shell,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cache::bootstrap()?;

    match Cli::parse().command {
        CliCommands::Install { v } => {
            Commands::install_version(&v)?;
        }
        CliCommands::Uninstall { v } => {
            Commands::uninstall_version(&v)?;
        }
        CliCommands::Use { v } => {
            Commands::use_version(&v)?;
        }
        CliCommands::List => {
            Commands::list_versions()?;
        }
        CliCommands::ListRemote => {
            Commands::list_remote_versions()?;
        }
        CliCommands::Shell => {
            Commands::display_shell_instructions()?;
        }
    }
    Ok(())
}
