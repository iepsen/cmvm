use clap::{Parser, Subcommand};

mod cache;
mod commands;
mod config;
mod constants;
mod http;
mod package;
mod platform;
mod releases;
mod versions;

use crate::config::Config;
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
    Install { version: String },

    /// Uninstall a cmake version
    Uninstall { version: String },

    /// Use a cmake version
    Use { version: String },

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
            Commands::list_versions()?;
        }
        CliCommands::ListRemote => {
            Commands::list_remote_versions()?;
        }
        CliCommands::Shell => {
            let config = Config::new();
            let current_version_dir = config.get_current_version_dir()?;

            println!(
                "[cmvm] {} {} {} {}\n\n  {} {}",
                "When `cmvm use <version>` is invoked, it changes the `current`",
                "symbolic link to the right cmake binary path. As cmvm doesn't",
                "manage the `current` path in the system, it requires to",
                "manually add it to the $PATH:",
                "export PATH=\"{}/bin:$PATH\"",
                current_version_dir.to_string_lossy()
            );
        }
    }
    Ok(())
}
