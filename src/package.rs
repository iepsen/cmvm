use anyhow::{bail, Result};
use flate2::read::GzDecoder;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use tar::Archive;
extern crate fs_extra;
use crate::http;
use crate::storage::Storage;
use crate::versions::{Asset, Version};
use crate::{cache, platform};
use fs_extra::dir;

const MAX_COMPRESSED_ARCHIVE_SIZE: u64 = 512 * 1024 * 1024;
const MAX_UNCOMPRESSED_ARCHIVE_SIZE: u64 = 2 * 1024 * 1024 * 1024;
const MAX_ARCHIVE_ENTRIES: usize = 100_000;
const MAX_ARCHIVE_METADATA_SIZE: u64 = 1024 * 1024;

pub fn get_cmake_release(version: &Version, storage: &impl Storage) -> Result<()> {
    let assets = filter_platform_assets(version);

    if assets.is_empty() {
        bail!("[cmvm] No asset found.");
    }

    let asset = assets.first();

    if let Some(asset) = asset {
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
    let cache_dir = storage.get_cache_dir()?;
    let version_dir_path = cache_dir.join(tag_name);

    if version_dir_path.exists() {
        cache::delete(&version_dir_path)?;
    }

    cache::create_dir(&version_dir_path)?;

    println!("[cmvm] Downloading {}.", asset.browser_download_url);
    let response = http::get(&asset.browser_download_url)?;
    if response
        .content_length()
        .is_some_and(|size| size > MAX_COMPRESSED_ARCHIVE_SIZE)
    {
        bail!("[cmvm] Download exceeds the maximum archive size.");
    }

    let file_path = &cache_dir.join(tag_name).join(&asset.name);
    let mut file = cache::create_file(file_path)?;
    let mut response = response.take(MAX_COMPRESSED_ARCHIVE_SIZE + 1);
    if io::copy(&mut response, &mut file)? > MAX_COMPRESSED_ARCHIVE_SIZE {
        drop(file);
        fs::remove_file(file_path)?;
        bail!("[cmvm] Download exceeds the maximum archive size.");
    }

    Ok(())
}

fn uncompress(tag_name: &str, asset: &Asset, storage: &impl Storage) -> Result<()> {
    let cache_dir = storage.get_cache_dir()?;
    let archive_path = cache_dir.join(tag_name).join(&asset.name);
    validate_archive(
        GzDecoder::new(fs::File::open(&archive_path)?),
        MAX_ARCHIVE_ENTRIES,
        MAX_UNCOMPRESSED_ARCHIVE_SIZE,
        MAX_ARCHIVE_METADATA_SIZE,
    )?;
    let compressed_file = fs::File::open(archive_path)?;
    let gz = GzDecoder::new(compressed_file);

    println!("[cmvm] Uncompressing {}.", asset.name);
    unpack_archive(
        gz,
        &cache_dir.join(tag_name),
        MAX_ARCHIVE_ENTRIES,
        MAX_UNCOMPRESSED_ARCHIVE_SIZE,
    )?;

    Ok(())
}

fn validate_archive<R: Read>(
    reader: R,
    max_entries: usize,
    max_uncompressed_size: u64,
    max_metadata_size: u64,
) -> Result<()> {
    let mut archive = Archive::new(reader);
    let mut entry_count = 0;
    let mut uncompressed_size: u64 = 0;

    for entry in archive.entries()?.raw(true) {
        let entry = entry?;
        entry_count += 1;
        if entry_count > max_entries {
            bail!("[cmvm] Archive contains too many entries.");
        }

        let entry_size = entry.size();
        uncompressed_size = uncompressed_size.checked_add(entry_size).ok_or_else(|| {
            anyhow::anyhow!("[cmvm] Archive exceeds the maximum uncompressed size.")
        })?;
        if uncompressed_size > max_uncompressed_size {
            bail!("[cmvm] Archive exceeds the maximum uncompressed size.");
        }

        let entry_type = entry.header().entry_type();
        if (entry_type.is_gnu_longname()
            || entry_type.is_gnu_longlink()
            || entry_type.is_pax_local_extensions()
            || entry_type.is_pax_global_extensions())
            && entry_size > max_metadata_size
        {
            bail!("[cmvm] Archive metadata entry exceeds the maximum size.");
        }
    }

    Ok(())
}

fn unpack_archive<R: Read>(
    reader: R,
    destination: &Path,
    max_entries: usize,
    max_uncompressed_size: u64,
) -> Result<()> {
    let mut archive = Archive::new(reader);
    let mut entry_count = 0;
    let mut uncompressed_size: u64 = 0;

    for entry in archive.entries()? {
        let mut entry = entry?;
        entry_count += 1;
        if entry_count > max_entries {
            bail!("[cmvm] Archive contains too many entries.");
        }

        uncompressed_size = uncompressed_size.checked_add(entry.size()).ok_or_else(|| {
            anyhow::anyhow!("[cmvm] Archive exceeds the maximum uncompressed size.")
        })?;
        if uncompressed_size > max_uncompressed_size {
            bail!("[cmvm] Archive exceeds the maximum uncompressed size.");
        }

        if !entry.unpack_in(destination)? {
            bail!("[cmvm] Archive entry is outside the destination directory.");
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
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Cursor;

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

    fn make_archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let encoder = GzEncoder::new(Vec::new(), Compression::default());
        let mut builder = tar::Builder::new(encoder);
        for (path, contents) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, path, *contents).unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap()
    }

    fn make_metadata_archive(metadata: &[u8]) -> Vec<u8> {
        let encoder = GzEncoder::new(Vec::new(), Compression::default());
        let mut builder = tar::Builder::new(encoder);
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(tar::EntryType::GNULongName);
        header.set_size(metadata.len() as u64);
        header.set_cksum();
        builder
            .append_data(&mut header, "././@LongLink", metadata)
            .unwrap();
        builder.into_inner().unwrap().finish().unwrap()
    }

    fn make_destination(name: &str) -> std::path::PathBuf {
        let destination =
            std::env::temp_dir().join(format!("cmvm_package_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&destination);
        fs::create_dir_all(&destination).unwrap();
        destination
    }

    #[test]
    fn test_unpack_archive_rejects_too_many_entries() {
        let archive = make_archive(&[("first", b"first"), ("second", b"second")]);
        let destination = make_destination("entry_limit");

        let result = unpack_archive(GzDecoder::new(Cursor::new(archive)), &destination, 1, 100);

        assert!(result.is_err());
        let _ = fs::remove_dir_all(destination);
    }

    #[test]
    fn test_unpack_archive_rejects_expansion_over_limit() {
        let archive = make_archive(&[("file", b"content")]);
        let destination = make_destination("size_limit");

        let result = unpack_archive(GzDecoder::new(Cursor::new(archive)), &destination, 10, 6);

        assert!(result.is_err());
        assert!(!destination.join("file").exists());
        let _ = fs::remove_dir_all(destination);
    }

    #[test]
    fn test_validate_archive_rejects_oversized_metadata() {
        let archive = make_metadata_archive(b"longname");

        let result = validate_archive(GzDecoder::new(Cursor::new(archive)), 10, 100, 4);

        assert!(result.is_err());
    }

    #[test]
    fn test_unpack_archive_extracts_valid_entry() {
        let archive = make_archive(&[("file", b"content")]);
        let destination = make_destination("valid");

        validate_archive(GzDecoder::new(Cursor::new(archive.clone())), 10, 100, 100).unwrap();
        unpack_archive(GzDecoder::new(Cursor::new(archive)), &destination, 10, 100).unwrap();

        assert_eq!(fs::read(destination.join("file")).unwrap(), b"content");
        let _ = fs::remove_dir_all(destination);
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
