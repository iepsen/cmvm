use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::fs;
use tar::Archive;
extern crate fs_extra;
use crate::constants::{BASE_RELEASE_URL, CACHE_DIR, VERSIONS_DIR};
use crate::http;
use crate::platform::get_platform_info;
use crate::releases;
use crate::versions::{Asset, Version};
use crate::{cache, platform};
use fs_extra::dir;

#[derive(Serialize, Deserialize, Debug)]
struct DownloadDetail {
    os: Vec<String>,
    architecture: Vec<String>,
    class: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DownloadOptions {
    files: Vec<DownloadDetail>,
}

pub fn get_cmake_release(version: &Version) -> Result<(), Box<dyn std::error::Error>> {
    download_asset_json(version).expect("[cmvm] download_asset_json error");

    let release_version = cache::open_file(CACHE_DIR.join(format!("{}.json", version.tag_name)))?;
    let platform_info = get_platform_info()?;
    let download_options: DownloadOptions = serde_json::from_str(release_version.as_str())?;
    let download_detail = download_options
        .files
        .iter()
        .find(|f| f.os.iter().any(|detail| detail == &platform_info.name) && f.class == "archive");

    if let Some(download_detail) = download_detail {
        download(version, download_detail)?;
        uncompress(version, download_detail)?;
        copy(version, download_detail)?;
        clean(version)?;
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
                .find(|pattern| asset.name.contains(pattern.as_str()))
                != None
        })
        .collect()
}

fn download_asset_json(version: &Version) -> Result<(), Box<dyn std::error::Error>> {
    let asset = releases::get_release_asset(version)?;
    if let Some(asset) = asset {
        let mut response = http::get(asset.browser_download_url.as_str())?;
        let file_path = &CACHE_DIR.join(format!("{}.json", version.tag_name));
        let mut file = cache::create_file(file_path)?;
        response.copy_to(&mut file)?;
    }
    Ok(())
}

fn download(
    version: &Version,
    download_detail: &DownloadDetail,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = CACHE_DIR.join(&version.tag_name);

    if path.exists() {
        cache::delete(Some(&path))?;
    }

    cache::create_dir(Some(&path))?;

    let package_name = download_detail.name.as_str();
    let package_url = format!(
        "{}/{}{}/{}",
        BASE_RELEASE_URL, "v", version.tag_name, package_name
    );

    println!("[cmvm] Downloading {}.", package_url);
    let mut response = http::get(package_url.as_str())?;
    let file_path = &CACHE_DIR.join(&version.tag_name).join(package_name);
    let mut file = cache::create_file(file_path)?;
    response.copy_to(&mut file)?;

    Ok(())
}

fn uncompress(
    version: &Version,
    download_detail: &DownloadDetail,
) -> Result<(), Box<dyn std::error::Error>> {
    let compressed_file = fs::read(
        CACHE_DIR
            .join(&version.tag_name)
            .join(&download_detail.name),
    )?;

    let gz = GzDecoder::new(&*compressed_file);
    let mut archive = Archive::new(gz);

    println!("[cmvm] Uncompressing {}.", download_detail.name);
    archive.unpack(&CACHE_DIR.join(&version.tag_name))?;

    Ok(())
}

fn copy(
    version: &Version,
    download_detail: &DownloadDetail,
) -> Result<(), Box<dyn std::error::Error>> {
    let cmake_cache_dir = &CACHE_DIR
        .join(&version.tag_name)
        .join(download_detail.name.replace(".tar.gz", ""))
        .join("CMake.app/Contents");

    let options = dir::CopyOptions::new();
    let mut from_paths: Vec<String> = Vec::new();

    for dir in vec!["bin", "doc", "man", "share"] {
        from_paths.push(
            cmake_cache_dir
                .join(dir)
                .into_os_string()
                .into_string()
                .unwrap(),
        );
    }

    if VERSIONS_DIR.join(&version.tag_name).exists() {
        cache::delete(Some(&VERSIONS_DIR.join(&version.tag_name)))?;
    }

    cache::create_dir(Some(&VERSIONS_DIR.join(&version.tag_name)))?;
    let destination_dir = VERSIONS_DIR.join(&version.tag_name);

    println!("[cmvm] Setting up {}.", &version.tag_name);
    fs_extra::copy_items(&from_paths, destination_dir, &options)?;

    Ok(())
}

fn clean(version: &Version) -> Result<(), Box<dyn std::error::Error>> {
    cache::delete(Some(&CACHE_DIR.join(&version.tag_name)))?;
    cache::delete(Some(&CACHE_DIR.join(format!("{}.json", version.tag_name))))?;
    println!("[cmvm] Cleaning cache.");
    Ok(())
}
