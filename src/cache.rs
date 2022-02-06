use std::fs;
use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;
use serde_json::{Value};
use serde::{Deserialize, Serialize};
use flate2::read::GzDecoder;
use tar::Archive;
extern crate fs_extra;
use fs_extra::dir;
use crate::http;


const BASE_URL: &str = "https://api.github.com/repos/Kitware/CMake/releases";
const BASE_RELEASE_URL: &str = "https://github.com/Kitware/CMake/releases/download";
const CACHE_FILE_NAME: &str = "releases.json";

#[derive(Serialize, Deserialize, Debug)]
struct OS {
  os: Vec<String>,
  architecture: Vec<String>,
  class: String,
  name: String
}

#[derive(Serialize, Deserialize, Debug)]
struct CMakeFiles {
  files: Vec<OS>,
}

pub fn generate_cache(page: Option<i32>) -> Result<(), Box<dyn std::error::Error>> {
  let current_page = page.unwrap_or(1);
  let first_page = current_page == 1;
  let mut response = http::get(format!("{}?page={}", BASE_URL, current_page).as_str())?;

  if response.status().is_success() {
    let cache_dir = get_cache_dir();
    let file_name = cache_dir.join(format!("{}.json", current_page));

    if file_name.exists() {
      if fs::remove_file(&file_name).is_err() {
        println!("[cmvm] Unable to remove file {:?}", file_name);
      }
    }

    let mut file = File::create(file_name)?;
    response.copy_to(&mut file)?;

    if first_page {
      if let Some(link_header) = response.headers().get("link") {
        let pages = get_number_of_pages(link_header.to_str().unwrap());
        for page in 2..pages.unwrap() + 1 {
          let result = generate_cache(Some(page));
          if result.is_err() {
            println!("[cmvm] Unable to generate cache for page {} with error {:?}", page, result.err());
          }
        }
        let merge_result = merge(pages.unwrap());
        if merge_result.is_err() {
          println!("[cmvm] Unable to merge with error {:?}", merge_result);
        }
      }
    }
  }
  Ok(())
}

fn merge(pages: i32) -> Result<(), Box<dyn std::error::Error>> {
  let mut releases: Vec<Value> = Vec::new();

  if get_file_path().exists() {
    if fs::remove_file(get_file_path()).is_err() {
      println!(
        "[cmvm] Unable to remove file {:?}", 
        get_file_path()
      );
    }
  }

  for page in 1..pages + 1 {
    let page_file_name = get_cache_dir()
      .join(format!("{}.json", page));

    if page_file_name.exists() {
      let page_file_contents = fs::read_to_string(&page_file_name).unwrap();
      let page_releases: Value = serde_json::from_str(page_file_contents.as_str())?;
      let page_releases_array = page_releases.as_array().unwrap();

      for page_release in page_releases_array {
        releases.push(page_release.clone());
      }

      if fs::remove_file(&page_file_name).is_err() {
        println!("[cmvm] Unable to remove intermediate cache file {:?}", page_file_name);
      }
    }
  }

  let mut cache_file = File::create(get_file_path())?;
  let cache_json = serde_json::to_string(&releases).unwrap();
  let cache_result = cache_file.write_all(cache_json.as_bytes());

  if cache_result.is_err() {
    println!("[cmvm] Unable to create cache file with error: {:?}", cache_result.err());
  }
  Ok(())
}

fn get_number_of_pages(link_header: &str) -> Result<i32, i32> {
  let parsed_link_header = parse_link_header::parse(link_header).unwrap();
  let last_link = parsed_link_header.get(&Some("last".to_string())).unwrap();
  let last_page = last_link.queries["page"].parse::<i32>().unwrap();
  return Ok(last_page);
}

pub fn bootstrap() {
  let home_dir: PathBuf = dirs::home_dir().unwrap();
  let cmvm_dir: PathBuf = home_dir.join(".cmvm");
  let cache_dir: PathBuf = cmvm_dir.join("cache");
  let versions_dir: PathBuf = cmvm_dir.join("versions");
  
  if !cmvm_dir.exists() {
    if fs::create_dir(cmvm_dir).is_err() {
      println!("[cmvm] Unable to create .cmvm dir");
      return;
    }
  }

  if !versions_dir.exists() {
    if fs::create_dir(versions_dir).is_err() {
      println!("[cmvm] Unable to create .cmvm/versions dir");
      return;
    }
  }
  
  if !cache_dir.exists() {
    if fs::create_dir(cache_dir).is_err() {
      println!("[cmvm] Unable to create .cmvm/cache dir");
      return;
    }
  }
}

pub fn get_cache_dir() -> PathBuf {
  let cmvm_dir: PathBuf = dirs::home_dir().unwrap().join(".cmvm");
  let cache_dir: PathBuf = cmvm_dir.join("cache");

  bootstrap();

  return cache_dir;
}

pub fn get_versions_dir() -> PathBuf {
  let cmvm_dir: PathBuf = dirs::home_dir().unwrap().join(".cmvm");
  let versions_dir: PathBuf = cmvm_dir.join("versions");

  bootstrap();

  return versions_dir;
}

pub fn get_file_path() -> PathBuf {
  let cache_dir = get_cache_dir();
  cache_dir.join(CACHE_FILE_NAME)
}

fn open_cache_file(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
  let mut cache_file = File::options().read(true).open(path)?;
  let mut contents = String::new();
  
  if cache_file.read_to_string(&mut contents).is_err() {
    println!("[cmvm] Cannot write to file");
  }
  return Ok(contents);
}

fn set_cache_file(name: &str, url: &str) -> Result<(), Box<dyn std::error::Error>> {
  let mut response = http::get(url)?;
  if !response.status().is_success() {
    println!("Hm");
  }
  let cache_dir = get_cache_dir();
  let file_path = cache_dir.join(name);

  let mut file = File::create(&file_path)?;
  response.copy_to(&mut file)?;
  Ok(())
}

pub fn get_cache() -> Result<String, Box<dyn std::error::Error>> {
  Ok(open_cache_file(get_file_path())?)
}

pub fn cmake_version(version: &str, url: &str, platform: &String) -> Result<(), Box<dyn std::error::Error>> {
  let file_name = format!("{}.json", version);
  let cache_result = set_cache_file(&file_name.as_str(), url);
  if cache_result.is_err() {
    println!("[cmvm] Fail to obtain cache file {:?}", cache_result.err());
  }
  let contents = open_cache_file(get_cache_dir().join(file_name.as_str()));
  let cmake_files: CMakeFiles = serde_json::from_str(contents.unwrap().as_str()).unwrap();

  for file in cmake_files.files {
    if file.class == "archive" && file.os.contains(platform) {
      let cmake_file_url = format!("{}/{}/{}", BASE_RELEASE_URL, version, file.name);
      
      println!("[cmvm] Downloading cmake {}... ", version);
      
      if set_cache_file(file.name.as_str(), cmake_file_url.as_str()).is_err() {
        println!("[cmvm] Failed to obtain cache for {}", cmake_file_url.as_str());
      }

      println!("[cmvm] Uncompressing cmake {}... ", version);

      let cache_root_version_dir = get_cache_dir().join(version);
      let tar_gz = File::open(get_cache_dir().join(&file.name))?;
      let tar = GzDecoder::new(tar_gz);
      let mut archive = Archive::new(tar);
      archive.unpack(&cache_root_version_dir)?;

      println!("[cmvm] Copy cmake {} files... ", version);
      
      let install_version = get_versions_dir().join(version);
      if !install_version.exists() {
        let created_dir = fs::create_dir(&install_version);
        if created_dir.is_err() {
          println!("[cmvm] Unable to create dir: {:?}", created_dir.err());
        }
      }
      
      let cmake_cache_dir = cache_root_version_dir
        .join(file.name.replace(".tar.gz", ""))
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

      let cmake_destination_dir = install_version.into_os_string().into_string().unwrap();
      fs_extra::copy_items(&from_paths, cmake_destination_dir, &options).unwrap();

      println!("[cmvm] Remove cmake {} cache files... ", version);

      let cmake_cache_delete_dir = vec!(
        cache_root_version_dir
          .into_os_string()
          .into_string()
          .unwrap()
        );

      fs_extra::remove_items(&cmake_cache_delete_dir).unwrap();

      println!("[cmvm] Clean up complete.");
      println!("[cmvm] Version {} installed successfully.", version);
      println!("[cmvm] Run \"cmvm use {}\" to set it as default cmake version.", version);
    }
  }

  Ok(())
}
