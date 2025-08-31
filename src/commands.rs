use crate::{package, platform::is_supported_platform, releases, versions::Version};
use crate::storage::{Storage};
use crate::types::BoxError;

pub struct Commands {}

impl Commands {
    pub fn install_version(v: &String, storage: &impl Storage) -> Result<(),        BoxError> {
        releases::build_cache(storage)?;

        let versions_dir = storage.get_versions_dir()?;

        if let Some(version) = releases::get_release(&v.trim().to_string(), storage)? {
            if versions_dir.join(&version.get_tag_name()).exists() {
                println!("[cmvm] Version {} already installed.", v);
                Commands::use_version(&v, storage)?;
                return Ok(());
            }

            if !is_supported_platform() {
                Err("Platform not supported.")?;
            }

            match package::get_cmake_release(&version, storage) {
                Ok(()) => {
                    println!(
                        "[cmvm] Version {} installed successfully.",
                        &version.get_tag_name()
                    );
                    Commands::use_version(&v, storage)?;
                    println!("[cmvm] Done.");
                }
                Err(e) => println!(
                    "[cmvm] Error while installing version {}: {}", version.get_tag_name(), e
                ),
            }
        } else {
            println!("[cmvm] Version {} not found.", v);
        }

        Ok(())
    }

    pub fn uninstall_version(v: &String, storage: &impl Storage) -> Result<(), BoxError> {
        match releases::delete_cache_release(v, storage) {
            Ok(()) => println!("[cmvm] Version {} uninstalled successfully.", v),
            Err(e) => println!("[cmvm] Version {} is not installed. {}", v, e),
        }
        Ok(())
    }

    pub fn list_remote_versions(storage: &impl Storage) -> Result<(), BoxError> {
        releases::build_cache(storage)?;

        println!("[cmvm] List of available versions to install:");
        match Version::list_remote(storage) {
            Ok(versions) => println!("{}", versions),
            Err(_) => println!("[cmvm] There is no versions installed yet."),
        };
        Ok(())
    }

    pub fn list_versions(storage: &impl Storage) -> Result<(), BoxError> {
        match Version::list(storage) {
            Ok(versions) => {
                if versions.len() > 0 {
                    println!("[cmvm] Installed versions:");
                    println!("{}", versions);
                } else {
                    println!("[cmvm] No cmake versions installed yet. Use `cmvm install <version>` to install your first cmake version.");
                    println!("[cmvm] Type `cmvm help` for more information.");
                }
            }
            Err(_) => println!("[cmvm] There is no versions installed yet."),
        };
        Ok(())
    }

    pub fn use_version(v: &str, storage: &impl Storage) -> Result<(), BoxError> {
        if let Some(mut version) = releases::get_release(&v.trim().to_string(), storage)? {
            match version.r#use(storage) {
                Ok(_) => println!("[cmvm] Version {} set as default.", version.get_tag_name()),
                Err(e) => println!(
                    "[cmvm] Error when trying to set version {}: {}",
                    version.get_tag_name(),
                    e
                ),
            }
        } else {
            println!("[cmvm] Version {} not found.", v);
        }
        Ok(())
    }

    pub fn display_shell_instructions(storage: &impl Storage) -> Result<(), BoxError> {
        let current_version_dir = storage.get_current_version_dir()?;

        println!(
            "[cmvm] {} {} {} {}\n\n {}",
            "When `cmvm use <version>` is invoked, it changes the `current`",
            "symbolic link to the right cmake binary path. As cmvm doesn't",
            "manage the `current` path in the system, it requires to",
            "manually add it to the $PATH:",
            format!(
                "export PATH=\"{}/bin:$PATH\"",
                &current_version_dir.to_string_lossy()
            )
        );

        Ok(())
    }
}
