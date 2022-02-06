use std::fs;
use std::io::prelude::*;
use std::fs::File;
use std::thread;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::{Value};
use crate::http;

const BASE_URL: &str = "https://api.github.com/repos/Kitware/CMake/releases";
const CACHE_FILE_NAME: &str = "releases.json";

#[derive(Serialize, Deserialize, Debug)]
struct Asset {
  content_type: String,
  browser_download_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Version {
  tag_name: String,
  assets: Vec<Asset>
}

fn get_number_of_pages(link_header: &str) -> Result<i32, i32> {
  let parsed_link_header = parse_link_header::parse(link_header).unwrap();
  let last_link = parsed_link_header.get(&Some("last".to_string())).unwrap();
  let last_page = last_link.queries["page"].parse::<i32>().unwrap();
  return Ok(last_page);
}

fn generate_cache(page: Option<i32>) -> Result<(), Box<dyn std::error::Error>> {
  let current_page = page.unwrap_or(1);
  let first_page = current_page == 1;
  let mut response = http::get(format!("{}?page={}", BASE_URL, current_page).as_str())?;

  if response.status().is_success() {
    let (_, cache_dir, _) = get_cmvm_dirs();
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

fn bootstrap() {
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

fn get_cmvm_dirs() -> (PathBuf, PathBuf, PathBuf) {
  let cmvm_dir: PathBuf = dirs::home_dir().unwrap().join(".cmvm");
  let cache_dir: PathBuf = cmvm_dir.join("cache");
  let versions_dir: PathBuf = cmvm_dir.join("versions");

  bootstrap();

  return (cmvm_dir, cache_dir, versions_dir);
}

fn merge(pages: i32) -> Result<(), Box<dyn std::error::Error>> {
  let mut releases: Vec<Value> = Vec::new();
  let (_, cache_dir, _) = get_cmvm_dirs();
  let file_name = cache_dir.join(format!("{}", CACHE_FILE_NAME));

  if file_name.exists() {
    if fs::remove_file(&file_name).is_err() {
      println!("[cmvm] Unable to remove file {:?}", file_name);
    }
  }

  for page in 1..pages + 1 {
    let page_file_name = cache_dir.join(format!("{}.json", page));

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

  let mut cache_file = File::create(file_name)?;
  let cache_json = serde_json::to_string(&releases).unwrap();
  let cache_result = cache_file.write_all(cache_json.as_bytes());

  if cache_result.is_err() {
    println!("[cmvm] Unable to create cache file with error: {:?}", cache_result.err());
  }
  Ok(())
}

fn print_versions(versions: Vec<Version>) {

  println!("[cmvm] Available cmake versions:");
  for version in versions {
    println!("    {}", version.tag_name);
  }
}

pub fn get_versions() -> Result<(), Box<dyn std::error::Error>> {

  let (_, cache_dir, _) = get_cmvm_dirs();
  let file_name = cache_dir.join(CACHE_FILE_NAME);

  if !file_name.exists() {
    println!("[cmvm] Fetching versions at first time...");
    if generate_cache(None).is_err() {
      println!("[cmvm] Unable to generate cache");
    }
  } else {
    thread::spawn(|| {
      if generate_cache(None).is_err() {
        println!("[cmvm] Unable to generate cache");
      }
    });
  }

  let mut file = File::options().read(true).open(file_name)?;
  let mut contents = String::new();
  
  if file.read_to_string(&mut contents).is_err() {
    println!("[cmvm] Cannot write to file");
  }

  let raw_versions: Vec<Value> = serde_json::from_str(contents.as_str()).unwrap();
  let mut versions: Vec<Version> = Vec::new();
  for raw_version in raw_versions {
    if raw_version["tag_name"].as_str().unwrap().len() > 0 {
      let version: Version = serde_json::from_value(raw_version).unwrap();
      versions.push(version);
    }
  }

  print_versions(versions);

  Ok(())
}