use crate::cache;
use crate::constants::{CACHE_DIR, CURRENT_VERSION, RELEASES_FILE_NAME, VERSIONS_DIR};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Asset {
    pub content_type: String,
    pub browser_download_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Version {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

pub fn use_version(version: &Version) -> Result<(), Box<dyn std::error::Error>> {
    if CURRENT_VERSION.exists() {
        cache::delete(Some(&CURRENT_VERSION))?;
    }

    std::os::unix::fs::symlink(
        VERSIONS_DIR.join(&version.tag_name),
        CURRENT_VERSION.as_path(),
    )?;

    Ok(())
}

pub fn list_versions() -> Result<String, Box<dyn std::error::Error>> {
    let versions = cache::ls(Some(&VERSIONS_DIR))?;
    let mut mapped_versions: Vec<String> = Vec::new();
    let current = CURRENT_VERSION.read_link()?;

    for version in versions {
        if version.is_dir() {
            let mut checked: &str = " ";
            if version == current {
                checked = "*";
            }

            if let Some(file_name) = version.file_name() {
                let version_name = file_name.to_string_lossy().replace("v", "");
                mapped_versions.push(format!("[cmvm] {} {}", checked, version_name));
            }
        }
    }
    Ok(mapped_versions.join("\n"))
}

pub fn list_remote_versions() -> Result<String, Box<dyn std::error::Error>> {
    let mut versions: Vec<String> = Vec::new();

    let releases = cache::open_file(CACHE_DIR.join(RELEASES_FILE_NAME))?;
    let raw_versions: Vec<Value> = serde_json::from_str(releases.as_str())?;
    for raw_version in raw_versions {
        if let Some(tag_name) = raw_version["tag_name"].as_str() {
            if tag_name.len() > 0 {
                let version: Version = serde_json::from_value(raw_version)?;
                versions.push(format!("[cmvm] {}", version.tag_name.replace("v", "")));
            }
        }
    }

    Ok(versions.join("\n"))
}
