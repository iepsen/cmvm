use flate2::read::GzDecoder;
use std::fs;
use tar::Archive;
extern crate fs_extra;
use crate::constants::{CACHE_DIR, VERSIONS_DIR};
use crate::http;
use crate::versions::{Asset, Version};
use crate::{cache, platform};
use fs_extra::dir;

pub fn get_cmake_release(version: &Version) -> Result<(), Box<dyn std::error::Error>> {
    let assets = filter_platform_assets(&version);

    if assets.len() == 0 {
        Err("[cmvm] No asset found.")?;
    }

    let asset = assets.first();

    if let Some(asset) = asset {
        download(&version.get_tag_name(), &asset)?;
        uncompress(&version.get_tag_name(), &asset)?;
        copy(&version.get_tag_name(), asset)?;
        clean(&version.get_tag_name())?;
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

fn download(tag_name: &String, asset: &Asset) -> Result<(), Box<dyn std::error::Error>> {
    let path = CACHE_DIR.join(tag_name);

    if path.exists() {
        cache::delete(Some(&path))?;
    }

    cache::create_dir(Some(&path))?;

    println!("[cmvm] Downloading {}.", asset.browser_download_url);
    let mut response = http::get(&asset.browser_download_url)?;
    let file_path = &CACHE_DIR.join(tag_name).join(&asset.name);
    let mut file = cache::create_file(file_path)?;
    response.copy_to(&mut file)?;

    Ok(())
}

fn uncompress(tag_name: &String, asset: &Asset) -> Result<(), Box<dyn std::error::Error>> {
    let compressed_file = fs::read(CACHE_DIR.join(tag_name).join(&asset.name))?;

    let gz = GzDecoder::new(&*compressed_file);
    let mut archive = Archive::new(gz);

    println!("[cmvm] Uncompressing {}.", asset.name);
    archive.unpack(&CACHE_DIR.join(tag_name))?;

    Ok(())
}

fn copy(tag_name: &String, asset: &Asset) -> Result<(), Box<dyn std::error::Error>> {
    let base_path = &CACHE_DIR
        .join(tag_name)
        .join(asset.name.replace(".tar.gz", ""));

    let cmake_cache_dir = match base_path.join("CMake.app/Contents").exists() {
        true => base_path.join("CMake.app/Contents"),
        false => base_path.to_path_buf(),
    };

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

    if VERSIONS_DIR.join(tag_name).exists() {
        cache::delete(Some(&VERSIONS_DIR.join(tag_name)))?;
    }

    cache::create_dir(Some(&VERSIONS_DIR.join(tag_name)))?;
    let destination_dir = VERSIONS_DIR.join(tag_name);

    fs_extra::copy_items(&from_paths, destination_dir, &options)?;
    println!("[cmvm] Setting up {}.", tag_name);

    Ok(())
}

fn clean(tag_name: &String) -> Result<(), Box<dyn std::error::Error>> {
    cache::delete(Some(&CACHE_DIR.join(tag_name)))?;
    println!("[cmvm] Cleaning cache.");
    Ok(())
}
