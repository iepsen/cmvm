use clap::{Parser, Subcommand};

mod cache;
mod commands;
mod constants;
mod http;
mod package;
mod platform;
mod releases;
mod versions;

#[derive(Parser)]
#[clap(version, about = "cmake version manager")]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
        Commands::Install { version } => {
            commands::install_version(&version)?;
        }
        Commands::Uninstall { version } => {
            commands::uninstall_version(&version)?;
        }
        Commands::Use { version } => {
            commands::use_version(&version)?;
        }
        Commands::List => {
            commands::list_versions()?;
        }
        Commands::ListRemote => {
            commands::list_remote_versions()?;
        }
    }
    Ok(())
}
