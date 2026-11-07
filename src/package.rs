use anyhow::{bail, Result};
use flate2::read::GzDecoder;
use std::fs;
use std::io::Read;
use std::path::{Component, Path};
use tar::Archive;
extern crate fs_extra;
use crate::http;
use crate::storage::Storage;
use crate::versions::{Asset, Version};
use crate::{cache, platform};
use fs_extra::dir;

pub fn get_cmake_release(version: &Version, storage: &impl Storage) -> Result<()> {
    let tag_name = version.get_tag_name();
    validate_path_component(&tag_name, "version tag")?;

    let assets = filter_platform_assets(version);

    if assets.is_empty() {
        bail!("[cmvm] No asset found.");
    }

    let asset = assets.first();

    if let Some(asset) = asset {
        validate_path_component(&asset.name, "asset name")?;
        download(&tag_name, asset, storage)?;
        uncompress(&tag_name, asset, storage)?;
        copy(&tag_name, asset, storage)?;
        clean(&tag_name, storage)?;
    }

    Ok(())
}

fn validate_path_component(value: &str, description: &str) -> Result<()> {
    let mut components = Path::new(value).components();
    if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
        bail!("Invalid {description}: {value}");
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

    Ok(())
}

fn uncompress(tag_name: &str, asset: &Asset, storage: &impl Storage) -> Result<()> {
    let cache_dir = storage.get_cache_dir()?;
    let compressed_file = fs::read(cache_dir.join(tag_name).join(&asset.name))?;

    let gz = GzDecoder::new(&*compressed_file);
    let mut archive = Archive::new(gz);

    println!("[cmvm] Uncompressing {}.", asset.name);
    unpack_archive(&mut archive, &cache_dir.join(tag_name))?;

    Ok(())
}

fn unpack_archive<R: Read>(archive: &mut Archive<R>, destination: &Path) -> Result<()> {
    archive.set_mask(0o7022);
    archive.set_preserve_permissions(false);
    archive.set_preserve_ownerships(false);
    archive.set_unpack_xattrs(false);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_type = entry.header().entry_type();
        if !entry_type.is_file() && !entry_type.is_contiguous() && !entry_type.is_dir() {
            bail!("Unsupported entry type in archive.");
        }

        let path = entry.path()?.into_owned();
        if !entry.unpack_in(destination)? {
            bail!(
                "Archive entry {} is outside the destination.",
                path.display()
            );
        }
    }

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
    use std::env;
    use std::path::PathBuf;
    use tar::{Builder, EntryType, Header};

    fn make_asset(name: &str, content_type: &str) -> Asset {
        Asset {
            name: name.to_string(),
            content_type: content_type.to_string(),
            browser_download_url: "https://fake-url".to_string(),
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

    fn test_dir(name: &str) -> PathBuf {
        let path = env::temp_dir().join(format!("cmvm_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn archive_with_file(path: &str, mode: u32) -> Vec<u8> {
        let mut builder = Builder::new(Vec::new());
        let mut header = Header::new_gnu();
        header.set_size(1);
        header.set_mode(mode);
        header.set_cksum();
        builder.append_data(&mut header, path, &b"x"[..]).unwrap();
        builder.into_inner().unwrap()
    }

    #[test]
    fn test_validate_path_component_rejects_paths() {
        assert!(validate_path_component("3.22.0", "version tag").is_ok());
        assert!(validate_path_component("../outside", "version tag").is_err());
        assert!(validate_path_component("/outside", "asset name").is_err());
    }

    #[test]
    fn test_unpack_archive_rejects_entries_outside_destination() {
        let base = test_dir("archive_traversal");
        let destination = base.join("extract");
        fs::create_dir(&destination).unwrap();
        let mut builder = Builder::new(Vec::new());
        let mut header = Header::new_gnu();
        header.as_mut_bytes()[..10].copy_from_slice(b"../outside");
        header.set_size(1);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, &b"x"[..]).unwrap();
        let archive_data = builder.into_inner().unwrap();
        let mut archive = Archive::new(&archive_data[..]);

        assert!(unpack_archive(&mut archive, &destination).is_err());
        assert!(!base.join("outside").exists());
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn test_unpack_archive_rejects_link_entries() {
        let base = test_dir("archive_link");
        let destination = base.join("extract");
        fs::create_dir(&destination).unwrap();
        let mut builder = Builder::new(Vec::new());
        let mut header = Header::new_gnu();
        header.set_entry_type(EntryType::symlink());
        header.set_size(0);
        header.set_mode(0o777);
        header.set_cksum();
        builder
            .append_link(&mut header, "link", "../outside")
            .unwrap();
        let archive_data = builder.into_inner().unwrap();
        let mut archive = Archive::new(&archive_data[..]);

        assert!(unpack_archive(&mut archive, &destination).is_err());
        assert!(!base.join("outside").exists());
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    #[cfg(unix)]
    fn test_unpack_archive_masks_unsafe_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let base = test_dir("archive_permissions");
        let destination = base.join("extract");
        fs::create_dir(&destination).unwrap();
        let archive_data = archive_with_file("file", 0o7777);
        let mut archive = Archive::new(&archive_data[..]);

        unpack_archive(&mut archive, &destination).unwrap();

        let mode = fs::metadata(destination.join("file"))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777;
        assert_eq!(mode & 0o7022, 0);
        let _ = fs::remove_dir_all(base);
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
}
