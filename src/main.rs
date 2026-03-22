use clap::{Parser, Subcommand};

mod cache;
mod commands;
mod constants;
mod http;
mod package;
mod platform;
mod releases;
mod storage;
mod versions;

use crate::storage::StorageImpl;
use anyhow::Result;
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

fn main() -> Result<()> {
    let storage = StorageImpl::default();

    cache::bootstrap(&storage)?;

    match Cli::parse().command {
        CliCommands::Install { v } => Commands::install_version(&v, &storage)?,
        CliCommands::Uninstall { v } => Commands::uninstall_version(&v, &storage)?,
        CliCommands::Use { v } => Commands::use_version(&v, &storage)?,
        CliCommands::List => Commands::list_versions(&storage)?,
        CliCommands::ListRemote => Commands::list_remote_versions(&storage)?,
        CliCommands::Shell => Commands::display_shell_instructions(&storage)?,
    }
    Ok(())
}
