use std::{fs, io::Write, thread};
use serde_json::{Value};
use crate::constants::{BASE_URL, CACHE_DIR, RELEASES_FILE_NAME};
use crate::cache;
use crate::http;
use crate::versions::{Version, Asset};

pub fn build_cache() {
  if !CACHE_DIR.join(RELEASES_FILE_NAME).exists() {
    println!("[cmvm] Fetching versions at first time...");
    if generate_cache(None).is_err() {
      println!("[cmvm] Failed to fetch remote versions");
    }
  } else {
    thread::spawn(|| {
      if generate_cache(None).is_err() {
        println!("[cmvm] Failed to fetch remote versions");
      }  
    });
  }
}

pub fn get_release(version: String) -> Option<Version> {
  let releases = get_releases().unwrap();
  let release = releases
    .iter()
    .find(|v| 
      v.tag_name.replace("v", "") == version
  );

  if let Some(release) = release {
    return Some(release.clone());
  }
  return None;
}

pub fn get_release_asset(version: &Version) -> Result<Option<Asset>, Box<dyn std::error::Error>>{
  let mut asset: Option<Asset> = None;
  let releases_versions = get_releases().unwrap();
  
  for release_version in releases_versions {
    if release_version.tag_name == version.tag_name {
      for version_asset in release_version.assets {
        if version_asset.content_type == "application/json" {
          asset = Some(version_asset);
        }
      }
    }
  }
  Ok(asset)
}

fn generate_cache(page: Option<i32>) -> Result<(), Box<dyn std::error::Error>> {
  let current_page = page.unwrap_or(1);
  let first_page = current_page == 1;
  let mut response = http::get(format!("{}?page={}", BASE_URL, current_page).as_str())?;

  if response.status().is_success() {
    let current_page_file = CACHE_DIR.join(format!("{}.json", current_page));

    if current_page_file.exists() {
      cache::delete(Some(&current_page_file))?;
    }

    let mut file = cache::create_file(current_page_file.as_path())?;
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

  if CACHE_DIR.join(RELEASES_FILE_NAME).exists() {
    cache::delete(Some(&CACHE_DIR.join(RELEASES_FILE_NAME)))?;
  }

  for page in 1..pages + 1 {
    let page_file_name = CACHE_DIR.join(format!("{}.json", page));

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

  let mut cache_file = cache::create_file(&CACHE_DIR.join(RELEASES_FILE_NAME)).unwrap();
  let cache_json = serde_json::to_string(&releases).unwrap();
  let cache_result = cache_file.write(cache_json.as_bytes());

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

fn get_releases() -> Result<Vec<Version>, Box<dyn std::error::Error>> {
  let releases = cache::open_file(CACHE_DIR.join(RELEASES_FILE_NAME));
  let raw_versions: Vec<Value> = serde_json::from_str(releases.unwrap().as_str()).unwrap();
  let mut versions: Vec<Version> = Vec::new();
  for raw_version in raw_versions {
    if raw_version["tag_name"].as_str().unwrap().len() > 0 {
      let version: Version = serde_json::from_value(raw_version).unwrap();
      versions.push(version);
    }
  }
  Ok(versions)
}
