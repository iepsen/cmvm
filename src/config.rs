use std::path::PathBuf;
use anyhow::{anyhow, Result};
use directories::ProjectDirs;

#[derive(Debug)]
pub struct Config {
    dirs: Option<ProjectDirs>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            dirs: ProjectDirs::from("com", "iepsen", "cmvm")
        }
    }

    pub fn get_data_dir(&self) ->Result<PathBuf> {
        if let Some(dirs) = &self.dirs {
            Ok(PathBuf::from(dirs.data_dir()))
        } else {
            Err(anyhow!("Data dir not found"))
        }
    }

    pub fn get_cache_dir(&self) -> Result<PathBuf> {
        if let Some(dirs) = &self.dirs {
            Ok(dirs.cache_dir().to_path_buf())
        } else {
            Err(anyhow!("Cache dir not found"))
        }
    }

    pub fn get_current_version_dir(&self) -> Result<PathBuf> {
        if let Some(dirs) = &self.dirs {
            Ok(dirs.data_dir().join("current"))
        } else {
            Err(anyhow!("Current version dir not found"))
        }
    }

    pub fn get_versions_dir(&self) -> Result<PathBuf> {
        if let Some(dirs) = &self.dirs {
            Ok(dirs.data_dir().join("versions"))
        } else {
            Err(anyhow!("Versions dir not found"))
        }
    }
}