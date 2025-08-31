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
        match &self.dirs {
            Some(dirs) => Ok(PathBuf::from(dirs.cache_dir())),
            None => Err(anyhow!("No cache dir found")),
        }
    }

    fn get_data_dir(&self) -> Result<PathBuf> {
        match &self.dirs {
            Some(dirs) => Ok(PathBuf::from(dirs.data_dir())),
            None => Err(anyhow!("No data dir found")),
        }
    }

    fn get_current_version_dir(&self) -> Result<PathBuf> {
        match &self.dirs {
            Some(dirs) => Ok(dirs.data_dir().join("current")),
            None => Err(anyhow!("No current dir found")),
        }
    }

    fn get_versions_dir(&self) -> Result<PathBuf> {
        match &self.dirs {
            Some(dirs) => Ok(dirs.data_dir().join("versions")),
            None => Err(anyhow!("No versions dir found")),
        }
    }
}
