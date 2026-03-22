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

impl StorageImpl {
    fn get_project_dirs(&self) -> Result<&ProjectDirs> {
        self.dirs.as_ref().ok_or_else(|| anyhow!("No project dirs found"))
    }
}

impl Storage for StorageImpl {
    fn get_cache_dir(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(self.get_project_dirs()?.cache_dir()))
    }

    fn get_data_dir(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(self.get_project_dirs()?.data_dir()))
    }

    fn get_current_version_dir(&self) -> Result<PathBuf> {
        Ok(self.get_project_dirs()?.data_dir().join("current"))
    }

    fn get_versions_dir(&self) -> Result<PathBuf> {
        Ok(self.get_project_dirs()?.data_dir().join("versions"))
    }
}
