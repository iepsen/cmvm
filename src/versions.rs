use crate::constants::{CACHE_DIR, CURRENT_VERSION, RELEASES_FILE_NAME, VERSIONS_DIR};
use crate::{cache, package, platform};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Asset {
    pub name: String,
    pub content_type: String,
    pub browser_download_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Version {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

impl Version {
    pub fn get_tag_name(&self) -> String {
        self.tag_name.replace("v", "")
    }

    pub fn set_tag_name(&mut self, tag_name: String) {
        self.tag_name = tag_name.to_string();
    }

    pub fn from_raw_value(raw_value: Value) -> Result<Version, Box<dyn std::error::Error>> {
        let version: Version = serde_json::from_value(raw_value)?;
        Ok(version)
    }

    pub fn r#use(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if CURRENT_VERSION.exists() {
            cache::delete(Some(&CURRENT_VERSION))?;
        }

        std::os::unix::fs::symlink(
            VERSIONS_DIR.join(self.get_tag_name()),
            CURRENT_VERSION.as_path(),
        )?;

        Ok(())
    }

    pub fn list() -> Result<String, Box<dyn std::error::Error>> {
        let versions = cache::ls(Some(&VERSIONS_DIR))?;
        let mut mapped_versions: Vec<String> = Vec::new();
        let current = CURRENT_VERSION.read_link().unwrap_or_default();

        for version in versions {
            if version.is_dir() {
                let mut checked: &str = " ";
                if version == current {
                    checked = "*";
                }

                if let Some(file_name) = version.file_name() {
                    let version_name = file_name.to_string_lossy();
                    mapped_versions.push(format!("[cmvm] {} {}", checked, version_name));
                }
            }
        }
        Ok(mapped_versions.join("\n"))
    }

    pub fn list_remote() -> Result<String, Box<dyn std::error::Error>> {
        let mut versions: Vec<String> = Vec::new();

        let releases = cache::open_file(CACHE_DIR.join(RELEASES_FILE_NAME))?;
        let raw_versions: Vec<Value> = serde_json::from_str(releases.as_str())?;
        for raw_version in raw_versions {
            let version = Version::from_raw_value(raw_version)?;
            let tag_name = version.get_tag_name();

            // skip release canditate versions
            if tag_name.contains("-rc") {
                continue;
            }

            let supported_definition = platform::supported_definition();
            let major_version = &version.get_major_version();

            // skip releases that doesn't match the required major version
            if major_version < &supported_definition.major_version_required {
                continue;
            }

            let assets: Vec<&Asset> = package::filter_platform_assets(&version);

            if assets.len() > 0 {
                versions.push(format!("[cmvm] {}", tag_name));
            }
        }
        Ok(versions.join("\n"))
    }

    pub fn get_major_version(&self) -> i32 {
        let tag_name = &mut self.get_tag_name();
        let mut split_version = tag_name.split(".");
        split_version.next().unwrap().parse::<i32>().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_version_tag_omits_v_key_word() {
        let version = Version {
            tag_name: "v1.10".to_string(),
            assets: vec![],
        };

        assert_eq!(version.get_tag_name(), "1.10");
    }
}
