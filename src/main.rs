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
            let home_dir = config.get_home_dir()?;
                
            println!(
                "[cmvm] {} {}\n",
                "In order to get cmvm working, you fist need to append cmake",
                "current version binary to the $PATH env variable."
            );
            
            println!("export PATH=\"{}/bin:$PATH\"\n", current_version_dir.to_string_lossy());

            println!(
                "[cmvm] {} {}/.bashrc or {}/.bash_profile",
                "For bash, the profile fie is",
                home_dir.to_string_lossy(), home_dir.to_string_lossy()
            );

            println!(
                "[cmvm] {} {}/.zshrc\n",
                "For zsh, you the profile is",
                home_dir.to_string_lossy()
            );

            println!("[cmvm] If you are unsure on what shell you use, you can type the following:\n");
            
            println!("echo $SHELL\n");
        }
    }
    Ok(())
}
