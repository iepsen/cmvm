use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use std::path::PathBuf;

pub(crate) trait Storage {
    fn get_cache_dir(&self) -> Result<PathBuf>;
    fn get_data_dir(&self) -> Result<PathBuf>;
    fn get_current_version_dir(&self) -> Result<PathBuf>;
    fn get_versions_dir(&self) -> Result<PathBuf>;
}
#[derive(Debug, Clone)]
pub struct StorageImpl {
    dirs: Option<ProjectDirs>,
}

impl Default for StorageImpl {
    fn default() -> Self {
        Self {
            dirs: ProjectDirs::from("com", "iepsen", "cmvm"),
        }
    }
}

impl Storage for StorageImpl {
    fn get_cache_dir(&self) -> Result<PathBuf> {
        if let Some(dirs) = &self.dirs {
            Ok(dirs.cache_dir().to_path_buf())
        } else {
            Err(anyhow!("Cache dir not found"))
        }
    }

    fn get_data_dir(&self) -> Result<PathBuf> {
        if let Some(dirs) = &self.dirs {
            Ok(PathBuf::from(dirs.data_dir()))
        } else {
            Err(anyhow!("Data dir not found"))
        }
    }

    fn get_current_version_dir(&self) -> Result<PathBuf> {
        if let Some(dirs) = &self.dirs {
            Ok(dirs.data_dir().join("current"))
        } else {
            Err(anyhow!("Current version dir not found"))
        }
    }

    fn get_versions_dir(&self) -> Result<PathBuf> {
        if let Some(dirs) = &self.dirs {
            Ok(dirs.data_dir().join("versions"))
        } else {
            Err(anyhow!("Versions dir not found"))
        }
    }
}
