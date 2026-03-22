use anyhow::{bail, Result};
use crate::constants::{BASE_URL, RELEASES_FILE_NAME};
use crate::http;
use crate::versions::Version;
use crate::{cache};
use serde_json::Value;
use std::{fs, io::Write};
use std::path::PathBuf;
use std::thread::spawn;
use crate::storage::Storage;

pub fn build_cache(storage: &impl Storage) -> Result<()> {
    let cache_dir = storage.get_cache_dir()?;
    let data_dir = storage.get_data_dir()?;

    match cache_dir.join(RELEASES_FILE_NAME).exists() {
        true => {
            spawn(|| {
                if cache_releases(cache_dir, data_dir, None).is_err() {
                    println!("[cmvm] Failed to fetch remote versions");
                }
            });
        }
        false => {
            println!("[cmvm] Fetching versions for the first time...");
            if cache_releases(cache_dir, data_dir, None).is_err() {
                println!("[cmvm] Failed to fetch remote versions");
            }
        }
    }
    Ok(())
}

pub fn get_release(version: &String, storage: &impl Storage) -> Result<Option<Version>> {
    let releases = Version::all_from_cache(storage)?;
    let release = releases.iter().find(|v| &v.get_tag_name() == version);

    match release {
        Some(release) => {
            let mut release_found: Version = release.clone();
            release_found.tag_name = release.get_tag_name();
            Ok(Some(release_found))
        },
        None => Ok(None)
    }
}

pub fn delete_cache_release(version: &String, storage: &impl Storage) -> Result<()> {
    let versions_dir = storage.get_versions_dir()?;
    let current_version_dir = storage.get_current_version_dir()?;
    if let Some(release) = get_release(version, storage)? {
        let version_path = versions_dir.join(release.get_tag_name());
        if current_version_dir.read_link()? == version_path {
            cache::delete(&current_version_dir)?;
        }
        cache::delete(&version_path.as_path())?;
    }

    Ok(())
}

fn cache_releases(cache_dir: PathBuf, data_dir: PathBuf, page: Option<i32>) -> Result<()> {
    let current_page = page.unwrap_or(1);
    let first_page = current_page == 1;
    let mut response = http::get(format!("{}?page={}", BASE_URL, current_page).as_str())?;

    if !response.status().is_success() {
        bail!("[cmvm] Something went wrong");
    }

    let current_page_file = cache_dir.join(format!("{}.json", current_page));

    if current_page_file.exists() {
        cache::delete(&current_page_file)?;
    }

    let mut file = cache::create_file(current_page_file.as_path(), data_dir.as_path())?;
    response.copy_to(&mut file)?;

    if first_page {
        if let Some(link_header) = response.headers().get("link") {
            let pages = get_number_of_pages(link_header.to_str()?)?;
            for page in 2..=pages {
                cache_releases(cache_dir.clone(), data_dir.clone(), Some(page))?;
            }
            merge(cache_dir.clone(), data_dir, pages)?;
        }
    }
    Ok(())
}

fn merge(cache_dir: PathBuf, data_dir: PathBuf, pages: i32) -> Result<()> {
    let mut releases: Vec<Value> = Vec::new();

    if cache_dir.join(RELEASES_FILE_NAME).exists() {
        cache::delete(&cache_dir.join(RELEASES_FILE_NAME))?;
    }

    for page in 1..=pages {
        let page_file = cache_dir.join(format!("{}.json", page));

        if page_file.exists() {
            let file_contents = fs::read_to_string(&page_file)?;
            let releases_json: Value = serde_json::from_str(file_contents.as_str())?;

            releases_json
                .as_array()
                .unwrap()
                .iter()
                .for_each(|r| releases.push(r.clone()));

            fs::remove_file(&page_file)?;
        }
    }

    let mut cache_file = cache::create_file(&cache_dir.join(RELEASES_FILE_NAME), data_dir.as_path())?;
    let cache_json = serde_json::to_string(&releases)?;
    cache_file.write(cache_json.as_bytes())?;

    Ok(())
}

fn get_number_of_pages(link_header: &str) -> Result<i32> {
    let mut last_page = 1;
    let parsed_link_header = parse_link_header::parse(link_header)?;
    let last_link = parsed_link_header.get(&Some("last".to_string()));
    if let Some(last_link) = last_link {
        last_page = last_link.queries["page"].parse::<i32>()?;
    }
    Ok(last_page)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;
    use anyhow::Result;
    use serde_json::json;

    struct MockStorage {
        cache_dir: PathBuf,
    }

    impl Storage for MockStorage {
        fn get_cache_dir(&self) -> Result<PathBuf> {
            Ok(self.cache_dir.clone())
        }
        fn get_data_dir(&self) -> Result<PathBuf> {
            Ok(self.cache_dir.clone())
        }
        fn get_current_version_dir(&self) -> Result<PathBuf> {
            Ok(self.cache_dir.join("current"))
        }
        fn get_versions_dir(&self) -> Result<PathBuf> {
            Ok(self.cache_dir.join("versions"))
        }
    }

    #[test]
    fn test_releases() {
        let cache_dir = env::temp_dir().join("cmvm_test_releases");
        cache::create_dir(cache_dir.as_path()).unwrap();
        let cache_file = cache::create_file(
            &cache_dir.join(RELEASES_FILE_NAME),
            cache_dir.as_path()
        );

        let raw_release = json!([
            {
                "assets": [
                    {
                        "browser_download_url": "https://fake-url",
                        "content_type": "None",
                        "name": "cmake-4.1.0-macos-universal.dmg"
                    }
                ],
                "assets_url": "https://fake-url",
                "tag_name": "v4.1.0",
                "draft": false,
                "prerelease": false
            }
        ]);

        cache_file.unwrap().write(raw_release.to_string().as_bytes()).ok();

        let storage = MockStorage { cache_dir: cache_dir.clone() };
        let releases = Version::all_from_cache(&storage).unwrap();
        let release = &releases[0];

        cache::delete(&*cache_dir).ok();

        assert_eq!(releases.len(), 1);
        assert_eq!(release.get_tag_name(), "4.1.0");
        assert_eq!(release.prerelease, Some(false));
    }

    #[test]
    fn test_releases_is_rc() {
        let cache_dir = env::temp_dir().join("cmvm_test_releases_is_rc");
        cache::create_dir(cache_dir.as_path()).unwrap();
        let cache_file = cache::create_file(
            &cache_dir.join(RELEASES_FILE_NAME),
            cache_dir.as_path()
        );

        let raw_release = json!([
            {
                "assets": [
                    {
                        "browser_download_url": "https://fake-url",
                        "content_type": "None",
                        "name": "cmake-4.1.0-macos-universal.dmg"
                    }
                ],
                "assets_url": "https://fake-url",
                "tag_name": "v4.1.0",
                "draft": false,
                "prerelease": true
            }
        ]);

        cache_file.unwrap().write(raw_release.clone().to_string().as_bytes()).ok();

        let storage = MockStorage { cache_dir: cache_dir.clone() };
        let releases = Version::all_from_cache(&storage).unwrap();
        let release = &releases[0];

        cache::delete(&*cache_dir).ok();

        assert_eq!(releases.len(), 1);
        assert_eq!(release.get_tag_name(), "4.1.0");
        assert_eq!(release.prerelease, Some(true));
    }

    #[test]
    fn test_get_number_of_pages_returns_last_page() {
        let link_header = "<https://api.github.com/repos/Kitware/CMake/releases?page=2>; rel=\"next\", <https://api.github.com/repos/Kitware/CMake/releases?page=7>; rel=\"last\"";
        let pages = get_number_of_pages(link_header).unwrap();
        assert_eq!(pages, 7);
    }

    #[test]
    fn test_get_number_of_pages_returns_one_when_no_last_link() {
        let link_header = "<https://api.github.com/repos/Kitware/CMake/releases?page=1>; rel=\"prev\"";
        let pages = get_number_of_pages(link_header).unwrap();
        assert_eq!(pages, 1);
    }

    fn write_releases_cache(cache_dir: &std::path::Path, raw_release: &serde_json::Value) {
        cache::create_dir(cache_dir).unwrap();
        let mut cache_file = cache::create_file(
            &cache_dir.join(RELEASES_FILE_NAME),
            cache_dir,
        ).unwrap();
        cache_file.write(raw_release.to_string().as_bytes()).unwrap();
    }

    #[test]
    fn test_get_release_returns_matching_version() {
        let cache_dir = env::temp_dir().join("cmvm_test_get_release");
        let raw_releases = json!([
            {
                "assets": [
                    {
                        "browser_download_url": "https://fake-url",
                        "content_type": "application/gzip",
                        "name": "cmake-3.25.0-linux-x86_64.tar.gz"
                    }
                ],
                "tag_name": "v3.25.0",
                "prerelease": false
            }
        ]);
        write_releases_cache(&cache_dir, &raw_releases);

        let storage = MockStorage { cache_dir: cache_dir.clone() };
        let release = get_release(&"3.25.0".to_string(), &storage).unwrap();

        cache::delete(&*cache_dir).ok();

        assert!(release.is_some());
        assert_eq!(release.unwrap().get_tag_name(), "3.25.0");
    }

    #[test]
    fn test_get_release_returns_none_when_not_found() {
        let cache_dir = env::temp_dir().join("cmvm_test_get_release_none");
        let raw_releases = json!([
            {
                "assets": [],
                "tag_name": "v3.25.0",
                "prerelease": false
            }
        ]);
        write_releases_cache(&cache_dir, &raw_releases);

        let storage = MockStorage { cache_dir: cache_dir.clone() };
        let release = get_release(&"3.99.0".to_string(), &storage).unwrap();

        cache::delete(&*cache_dir).ok();

        assert!(release.is_none());
    }
}