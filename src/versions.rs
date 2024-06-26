use crate::constants::RELEASES_FILE_NAME;
use crate::{cache, package, platform, Config};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Asset {
    pub name: String,
    pub content_type: String,
    pub browser_download_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Version {
    pub major: Option<i32>,
    pub minor: Option<i32>,
    pub patch: Option<i32>,
    pub rc: Option<String>,
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

impl Version {
    pub fn get_tag_name(&self) -> String {
        self.tag_name.replace("v", "")
    }

    pub fn from_raw_value(raw_value: Value) -> Result<Version, Box<dyn std::error::Error>> {
        let mut version: Version = serde_json::from_value(raw_value)?;

        // normalizing "-rc" prefixed releases: 3.14.0-rc3 -> 3.14.0.rc3
        let tag_name = version.get_tag_name().replace("-", ".");
        let mut split_version = tag_name.split(".");

        let (major, minor, patch, rc) = (
            split_version.next().unwrap().parse::<i32>().unwrap(),
            split_version.next().unwrap().parse::<i32>().unwrap(),
            split_version.next().unwrap().parse::<i32>().unwrap(),
            split_version
                .next()
                .unwrap_or(&"")
                .parse::<String>()
                .unwrap(),
        );

        version.major = Some(major);
        version.minor = Some(minor);
        version.patch = Some(patch);
        version.rc = Some(rc.to_string());

        Ok(version)
    }

    pub fn r#use(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::new();
        let current_version_dir = config.get_current_version_dir()?;
        let versions_dir = config.get_versions_dir()?;
        if current_version_dir.exists() {
            cache::delete(&current_version_dir)?;
        }

        std::os::unix::fs::symlink(
            versions_dir.join(self.get_tag_name()),
            current_version_dir.as_path(),
        )?;

        Ok(())
    }

    pub fn list() -> Result<String, Box<dyn std::error::Error>> {
        let config = Config::new();
        let current_version_dir = config.get_current_version_dir()?;
        let versions_dir = config.get_versions_dir()?;

        let versions = cache::ls(&versions_dir)?;
        let mut mapped_versions: Vec<String> = Vec::new();
        let current = current_version_dir.read_link().unwrap_or_default();

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
        let config = Config::new();
        let cache_dir = config.get_cache_dir()?;
        let mut versions: Vec<Version> = Vec::new();

        let releases = cache::open_file(cache_dir.join(RELEASES_FILE_NAME))?;
        let raw_versions: Vec<Value> = serde_json::from_str(releases.as_str())?;
        for raw_version in raw_versions {
            let version = Version::from_raw_value(raw_version)?;

            // skip release canditate versions
            if version.is_rc() {
                continue;
            }

            let supported_definition = platform::supported_definition();

            // skip releases that doesn't match the required major version
            if &version.major.unwrap() < &supported_definition.major_version_required {
                continue;
            }

            let assets: Vec<&Asset> = package::filter_platform_assets(&version);

            if assets.len() > 0 {
                versions.push(version);
            }
        }

        versions.sort();

        let version_tags: Vec<String> = versions
            .into_iter()
            .map(|v| format!("[cmvm] {}", v.get_tag_name()))
            .collect();

        Ok(version_tags.join("\n"))
    }

    fn is_rc(&self) -> bool {
        self.tag_name.contains("-rc")
    }
}

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    #[test]
    fn test_version_tag_omits_v_key_word() {
        let version = Version {
            major: Some(1),
            minor: Some(10),
            patch: Some(0),
            rc: None,
            tag_name: "v1.10.0".to_string(),
            assets: vec![],
        };

        assert_eq!(version.get_tag_name(), "1.10.0");
    }

    #[test]
    fn test_major_version_extracted() {
        let version = Version {
            major: Some(3),
            minor: Some(20),
            patch: Some(10),
            rc: None,
            tag_name: "v3.20.10".to_string(),
            assets: vec![],
        };

        assert_eq!(version.major.unwrap(), 3);
    }

    #[test]
    fn test_raw_version_converted_to_version_struct() {
        let raw_asset = json!({
            "name": "cmake-3.22.3-linux-aarch64.tar.gz",
            "browser_download_url": "http://fake_browser_download_url",
            "content_type": "application/gzip",
        });
        let raw_version = json!({
            "tag_name": "v3.22.3",
            "assets": [raw_asset]
        });

        let version_from_raw = Version::from_raw_value(raw_version);
        let version = version_from_raw.unwrap();
        let assets = version.assets.first();

        assert_eq!(version.tag_name, "v3.22.3");
        assert_eq!(version.assets.len(), 1);
        assert_eq!(assets.is_some(), true);
        assert_eq!(assets.unwrap().name, "cmake-3.22.3-linux-aarch64.tar.gz");
        assert_eq!(
            assets.unwrap().browser_download_url,
            "http://fake_browser_download_url"
        );
        assert_eq!(assets.unwrap().content_type, "application/gzip");
    }

    #[test]
    fn test_raw_version_rc_converted_to_version_struct() {
        let raw_asset = json!({
            "name": "cmake-3.22.3-linux-aarch64.tar.gz",
            "browser_download_url": "http://fake_browser_download_url",
            "content_type": "application/gzip",
        });
        let raw_version = json!({
            "tag_name": "v3.22.3-rc5",
            "assets": [raw_asset]
        });

        let version_from_raw = Version::from_raw_value(raw_version);
        let version = version_from_raw.unwrap();

        assert_eq!(version.tag_name, "v3.22.3-rc5");
        assert_eq!(version.major, Some(3));
        assert_eq!(version.minor, Some(22));
        assert_eq!(version.patch, Some(3));
        assert_eq!(version.rc, Some("rc5".to_string()));
        assert_eq!(version.is_rc(), true);
    }
}
