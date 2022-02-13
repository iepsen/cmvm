use std::{io};
use serde_json::Value;
use serde::{Deserialize, Serialize};
use crate::{cache::{ls, delete, open_file}, constants::{VERSIONS_DIR, CURRENT_VERSION, CACHE_DIR, RELEASES_FILE_NAME}};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Asset {
  pub content_type: String,
  pub browser_download_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Version {
  pub tag_name: String,
  pub assets: Vec<Asset>
}

pub fn use_version(version: &Version) -> Result<(), io::Error> {
  if CURRENT_VERSION.exists() {
    delete(Some(&CURRENT_VERSION))?;
  }

  std::os::unix::fs::symlink(
    VERSIONS_DIR.join(&version.tag_name), 
    CURRENT_VERSION.as_path()
  )
}

pub fn list_versions() -> Result<String, io::Error> {
  let versions = ls(Some(&VERSIONS_DIR))?;
  let mut mapped_versions: Vec<String> = Vec::new();
  let current = CURRENT_VERSION.read_link().unwrap_or_default();

  for version in versions {
    if version.is_dir() {
      let mut checked: &str = " ";
      if version == current {
        checked = "*";
      }

      let version_name = version.file_name().unwrap().to_str().unwrap().replace("v", "");
      mapped_versions.push(format!("[cmvm] {} {}", checked, version_name));
    }
  }
  Ok(mapped_versions.join("\n"))
}

pub fn list_remote_versions() -> Result<String, io::Error> {
  let mut versions: Vec<String> = Vec::new();

  let releases = open_file(CACHE_DIR.join(RELEASES_FILE_NAME));
  let raw_versions: Vec<Value> = serde_json::from_str(releases.unwrap().as_str()).unwrap();
  for raw_version in raw_versions {
    if raw_version["tag_name"].as_str().unwrap().len() > 0 {
      let version: Version = serde_json::from_value(raw_version).unwrap();
      versions.push(format!("[cmvm] {}", version.tag_name.replace("v", "")));
    }
  }

  Ok(versions.join("\n"))
}