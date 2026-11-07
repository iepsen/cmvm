use anyhow::{bail, Result};
use flate2::read::GzDecoder;
use std::fs;
use std::io::{Read, Write};
use tar::Archive;
extern crate fs_extra;
use crate::http;
use crate::storage::Storage;
use crate::versions::{validate_path_component, Asset, Version};
use crate::{cache, platform};
use fs_extra::dir;
use sha2::{Digest, Sha256};

pub fn get_cmake_release(version: &Version, storage: &impl Storage) -> Result<()> {
    validate_path_component(&version.get_tag_name())?;
    let assets = filter_platform_assets(version);

    if assets.is_empty() {
        bail!("[cmvm] No asset found.");
    }

    let asset = assets.first();

    if let Some(asset) = asset {
        validate_path_component(&asset.name)?;
        download(&version.get_tag_name(), asset, storage)?;
        uncompress(&version.get_tag_name(), asset, storage)?;
        copy(&version.get_tag_name(), asset, storage)?;
        clean(&version.get_tag_name(), storage)?;
    }

    Ok(())
}

pub fn filter_platform_assets(version: &Version) -> Vec<&Asset> {
    let supported_definitions = platform::supported_definition();
    version
        .assets
        .iter()
        .filter(|asset| {
            supported_definitions
                .content_types
                .contains(&asset.content_type)
        })
        .filter(|asset| {
            supported_definitions
                .name_contains
                .iter()
                .any(|pattern| asset.name.contains(pattern.as_str()))
        })
        .collect()
}

fn download(tag_name: &str, asset: &Asset, storage: &impl Storage) -> Result<()> {
    let expected_digest = expected_sha256(asset)?;

    let cache_dir = storage.get_cache_dir()?;
    let version_dir_path = cache_dir.join(tag_name);

    if version_dir_path.exists() {
        cache::delete(&version_dir_path)?;
    }

    cache::create_dir(&version_dir_path)?;

    println!("[cmvm] Downloading {}.", asset.browser_download_url);
    let mut response = http::get(&asset.browser_download_url)?;
    let file_path = &cache_dir.join(tag_name).join(&asset.name);
    let mut file = cache::create_file(file_path)?;
    response.copy_to(&mut file)?;
    file.flush()?;
    verify_sha256(file_path, expected_digest)?;

    Ok(())
}

fn expected_sha256(asset: &Asset) -> Result<&str> {
    let digest = asset
        .digest
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("[cmvm] Missing SHA-256 digest for {}.", asset.name))?;
    digest
        .strip_prefix("sha256:")
        .filter(|digest| digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| anyhow::anyhow!("[cmvm] Invalid SHA-256 digest for {}.", asset.name))
}

fn verify_sha256(file_path: &std::path::Path, expected_digest: &str) -> Result<()> {
    let mut file = fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let actual_digest = format!("{:x}", hasher.finalize());
    if !actual_digest.eq_ignore_ascii_case(expected_digest) {
        bail!(
            "[cmvm] SHA-256 digest mismatch for {}.",
            file_path.display()
        );
    }

    Ok(())
}

fn uncompress(tag_name: &str, asset: &Asset, storage: &impl Storage) -> Result<()> {
    let cache_dir = storage.get_cache_dir()?;
    let compressed_file = fs::read(cache_dir.join(tag_name).join(&asset.name))?;

    let gz = GzDecoder::new(&*compressed_file);
    let mut archive = Archive::new(gz);

    println!("[cmvm] Uncompressing {}.", asset.name);
    archive.unpack(cache_dir.join(tag_name))?;

    Ok(())
}

fn copy(tag_name: &str, asset: &Asset, storage: &impl Storage) -> Result<()> {
    let cache_dir = storage.get_cache_dir()?;
    let versions_dir = storage.get_versions_dir()?;
    let base_path = &cache_dir
        .join(tag_name)
        .join(asset.name.replace(".tar.gz", ""));

    let cmake_cache_dir = match base_path.join("CMake.app/Contents").exists() {
        true => base_path.join("CMake.app/Contents"),
        false => base_path.to_path_buf(),
    };

    let options = dir::CopyOptions::new();
    let mut from_paths: Vec<String> = Vec::new();

    for dir in ["bin", "doc", "man", "share"] {
        from_paths.push(
            cmake_cache_dir
                .join(dir)
                .into_os_string()
                .into_string()
                .unwrap(),
        );
    }

    if versions_dir.join(tag_name).exists() {
        cache::delete(&versions_dir.join(tag_name))?;
    }

    cache::create_dir(&versions_dir.join(tag_name))?;
    let destination_dir = versions_dir.join(tag_name);

    fs_extra::copy_items(&from_paths, destination_dir, &options)?;
    println!("[cmvm] Setting up {}.", tag_name);

    Ok(())
}

fn clean(tag_name: &str, storage: &impl Storage) -> Result<()> {
    let cache_dir = storage.get_cache_dir()?;
    cache::delete(&cache_dir.join(tag_name))?;
    println!("[cmvm] Cleaning cache.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::versions::{Asset, Version};

    fn make_asset(name: &str, content_type: &str) -> Asset {
        Asset {
            name: name.to_string(),
            content_type: content_type.to_string(),
            browser_download_url: "https://fake-url".to_string(),
            digest: None,
        }
    }

    fn make_version(assets: Vec<Asset>) -> Version {
        Version {
            major: Some(3),
            minor: Some(22),
            patch: Some(0),
            prerelease: Some(false),
            tag_name: "v3.22.0".to_string(),
            assets,
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_filter_platform_assets_matches_linux_x86_64_lowercase() {
        let assets = vec![make_asset(
            "cmake-3.22.0-linux-x86_64.tar.gz",
            "application/gzip",
        )];
        let version = make_version(assets);
        let filtered = filter_platform_assets(&version);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "cmake-3.22.0-linux-x86_64.tar.gz");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_filter_platform_assets_matches_linux_x86_64_uppercase() {
        let assets = vec![make_asset(
            "cmake-3.22.0-Linux-x86_64.tar.gz",
            "application/gzip",
        )];
        let version = make_version(assets);
        let filtered = filter_platform_assets(&version);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_filter_platform_assets_excludes_wrong_content_type() {
        let assets = vec![make_asset(
            "cmake-3.22.0-linux-x86_64.tar.gz",
            "application/octet-stream",
        )];
        let version = make_version(assets);
        let filtered = filter_platform_assets(&version);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_filter_platform_assets_excludes_non_linux_assets() {
        let assets = vec![
            make_asset("cmake-3.22.0-macos-universal.tar.gz", "application/gzip"),
            make_asset("cmake-3.22.0-windows-x86_64.zip", "application/zip"),
        ];
        let version = make_version(assets);
        let filtered = filter_platform_assets(&version);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_filter_platform_assets_returns_only_matching_from_mixed_list() {
        let assets = vec![
            make_asset("cmake-3.22.0-linux-x86_64.tar.gz", "application/gzip"),
            make_asset("cmake-3.22.0-macos-universal.tar.gz", "application/gzip"),
            make_asset("cmake-3.22.0-windows-x86_64.zip", "application/zip"),
        ];
        let version = make_version(assets);
        let filtered = filter_platform_assets(&version);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "cmake-3.22.0-linux-x86_64.tar.gz");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_filter_platform_assets_matches_macos_asset() {
        let assets = vec![make_asset(
            "cmake-3.22.0-macos-universal.tar.gz",
            "application/gzip",
        )];
        let version = make_version(assets);
        let filtered = filter_platform_assets(&version);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_verify_sha256_accepts_matching_digest() {
        let file_path = std::env::temp_dir().join("cmvm_test_valid_digest");
        fs::write(&file_path, b"cmake archive").unwrap();

        let result = verify_sha256(
            &file_path,
            "2b3cd85eec6db7a1a418559af44527e89b7c6af8d91af4a9a20c8f13d355b565",
        );

        fs::remove_file(file_path).unwrap();
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_sha256_rejects_mismatched_digest() {
        let file_path = std::env::temp_dir().join("cmvm_test_invalid_digest");
        fs::write(&file_path, b"cmake archive").unwrap();

        let result = verify_sha256(&file_path, &"0".repeat(64));

        fs::remove_file(file_path).unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_expected_sha256_requires_valid_asset_digest() {
        let mut asset = make_asset("cmake.tar.gz", "application/gzip");
        assert!(expected_sha256(&asset).is_err());

        asset.digest = Some("sha256:invalid".to_string());
        assert!(expected_sha256(&asset).is_err());

        asset.digest = Some(format!("sha256:{}", "a".repeat(64)));
        assert_eq!(expected_sha256(&asset).unwrap(), "a".repeat(64));
    }
}
