use std::fs;

use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use tar::Archive;

use fs_extra::dir;
extern crate fs_extra;

use crate::constants::{CACHE_DIR, VERSIONS_DIR, BASE_RELEASE_URL};
use crate::cache::{open_file, create_file, create_dir, delete};
use crate::versions::Version;
use crate::{http};
use crate::releases::{get_release_asset};

#[derive(Serialize, Deserialize, Debug)]
struct DownloadDetail {
  os: Vec<String>,
  architecture: Vec<String>,
  class: String,
  name: String
}

#[derive(Serialize, Deserialize, Debug)]
struct DownloadOptions {
  files: Vec<DownloadDetail>,
}

fn download_asset_json(version: &Version) -> Result<(), Box<dyn std::error::Error>> {
  match get_release_asset(version) {
    Ok(asset) => {
      if let Some(asset) = asset {
        let mut response = http::get(asset.browser_download_url.as_str()).unwrap();
        if response.status().is_success() {
          let mut file = create_file(&CACHE_DIR.join(format!("{}.json", version.tag_name)))?;
          response.copy_to(&mut file)?;
        }
      }
    },
    Err(e) => println!("[cmvm] Error trying to get release url for version {}: {}", version.tag_name, e),
  }
  Ok(())
}

fn download(version: &Version, download_detail: &DownloadDetail) -> Result<(), Box<dyn std::error::Error>> {
  let path = CACHE_DIR.join(&version.tag_name);

  if path.exists() {
    delete(Some(&path))?;
  }

  create_dir(Some(&path))?;

  let package_name = download_detail.name.as_str();
  let package_url = format!("{}/{}/{}", BASE_RELEASE_URL, version.tag_name, package_name);
  let mut response = http::get(package_url.as_str()).unwrap();
  let mut file = create_file(&CACHE_DIR.join(&version.tag_name).join(package_name))?;
  response.copy_to(&mut file)?;

  Ok(())
}

fn uncompress(version: &Version, download_detail: &DownloadDetail) -> Result<(), Box<dyn std::error::Error>> {
  let compressed_file = fs::read(CACHE_DIR.join(&version.tag_name).join(&download_detail.name)).unwrap();
  let gz = GzDecoder::new(&*compressed_file);
  let mut archive = Archive::new(gz);
  archive.unpack(&CACHE_DIR.join(&version.tag_name))?;
  
  Ok(())
}

fn copy(version: &Version, download_detail: &DownloadDetail) -> Result<(), Box<dyn std::error::Error>> {
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
        .unwrap()
    );
  }

  if VERSIONS_DIR.join(&version.tag_name).exists() {
    delete(Some(&VERSIONS_DIR.join(&version.tag_name))).unwrap();
  }

  create_dir(Some(&VERSIONS_DIR.join(&version.tag_name))).unwrap();

  fs_extra::copy_items(&from_paths, VERSIONS_DIR.join(&version.tag_name), &options)?;

  Ok(())
}

fn clean(version: &Version) -> Result<(), Box<dyn std::error::Error>>{
  delete(Some(&CACHE_DIR.join(&version.tag_name)))?;
  delete(Some(&CACHE_DIR.join(format!("{}.json", version.tag_name))))?;
  Ok(())
}

pub fn get_cmake_release(version: &Version) -> Result<(), Box<dyn std::error::Error>> {
  match download_asset_json(version) {
    Ok(()) => {
      match open_file(CACHE_DIR.join(format!("{}.json", version.tag_name))) {
        Ok(release_version) => {
          let download_options: DownloadOptions = serde_json::from_str(release_version.as_str()).unwrap();
          let download_detail = download_options
          .files
          .iter()
          .find(|f| f.os.contains(&"macos".to_string()) && f.class == "archive");

          if let Some(download_detail) = download_detail {
            download(version, download_detail)?;
            uncompress(version, download_detail)?;
            copy(version, download_detail)?;
            clean(version)?;
          }
        },
        Err(e) => println!("[cmvm] Error {}", e),
      }
    },
    Err(e) => println!("[cmvm] Error {}", e),
  }
  Ok(())
}
