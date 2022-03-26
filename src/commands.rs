use crate::{Config, package, platform::is_supported_platform, releases, versions::Version};

pub struct Commands {}

impl Commands {
    pub fn install_version(v: &String) -> Result<(), Box<dyn std::error::Error>> {
        releases::build_cache()?;

        let config = Config::new();
        let versions_dir = config.get_versions_dir()?;

        if let Some(version) = releases::get_release(&v.trim().to_string())? {
            if versions_dir.join(&version.get_tag_name()).exists() {
                println!("[cmvm] Version {} already installed.", v);
                Commands::use_version(&v)?;
                return Ok(());
            }

            if !is_supported_platform() {
                Err("Platform not supported.")?;
            }

            match package::get_cmake_release(&version) {
                Ok(()) => {
                    println!(
                        "[cmvm] Version {} installed successfully.",
                        &version.get_tag_name()
                    );
                    Commands::use_version(&v)?;
                    println!("[cmvm] Done.");
                }
                Err(e) => println!(
                    "[cmvm] Error while installing version {}: {}",
                    version.get_tag_name(),
                    e
                ),
            }
        } else {
            println!("[cmvm] Version {} not found.", v);
        }

        Ok(())
    }

    pub fn uninstall_version(v: &String) -> Result<(), Box<dyn std::error::Error>> {
        match releases::delete_cache_release(v) {
            Ok(()) => println!("[cmvm] Version {} uninstalled successfully.", v),
            Err(e) => println!("[cmvm] Version {} is not installed. {}", v, e),
        }
        Ok(())
    }

    pub fn list_remote_versions() -> Result<(), Box<dyn std::error::Error>> {
        releases::build_cache()?;

        println!("[cmvm] List of available versions to install:");
        match Version::list_remote() {
            Ok(versions) => println!("{}", versions),
            Err(_) => println!("[cmvm] There is no versions installed yet."),
        };
        Ok(())
    }

    pub fn list_versions() -> Result<(), Box<dyn std::error::Error>> {
        match Version::list() {
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

    pub fn use_version(v: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut version) = releases::get_release(&v.trim().to_string())? {
            match version.r#use() {
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
}
