use crate::constants::{BASE_URL, RELEASES_FILE_NAME};
use crate::http;
use crate::versions::Version;
use crate::{cache};
use serde_json::Value;
use std::{fs, io::Write};
use std::path::PathBuf;
use std::thread::spawn;
use crate::storage::Storage;

type BoxError = Box<dyn std::error::Error>;

pub fn build_cache(storage: &impl Storage) -> Result<(), BoxError> {
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

pub fn get_release(version: &String, storage: &impl Storage) -> Result<Option<Version>, BoxError> {
    let releases = get_releases(storage)?;
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

pub fn delete_cache_release(version: &String, storage: &impl Storage) -> Result<(), BoxError> {
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

fn cache_releases(cache_dir: PathBuf, data_dir: PathBuf, page: Option<i32>) -> Result<(), BoxError> {
    let current_page = page.unwrap_or(1);
    let first_page = current_page == 1;
    let mut response = http::get(format!("{}?page={}", BASE_URL, current_page).as_str())?;

    if !response.status().is_success() {
        Err("[cmvm] Something went wrong")?;
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
                cache_releases(cache_dir.clone(), data_dir.clone(), Some(page)).expect("Failed");
            }
            merge(cache_dir.clone(), data_dir, pages)?;
        }
    }
    Ok(())
}

fn merge(cache_dir: PathBuf, data_dir: PathBuf, pages: i32) -> Result<(), BoxError> {
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

fn get_number_of_pages(link_header: &str) -> Result<i32, BoxError> {
    let mut last_page = 1;
    let parsed_link_header = parse_link_header::parse(link_header)?;
    let last_link = parsed_link_header.get(&Some("last".to_string()));
    if let Some(last_link) = last_link {
        last_page = last_link.queries["page"].parse::<i32>()?;
    }
    Ok(last_page)
}

fn get_releases(storage: &impl Storage) -> Result<Vec<Version>, BoxError> {
    let cache_dir = storage.get_cache_dir()?;
    let releases = cache::open_file(cache_dir.join(RELEASES_FILE_NAME));
    let raw_versions: Vec<Value> = serde_json::from_str(releases.unwrap().as_str())?;

    let versions = raw_versions
        .into_iter()
        .filter(|rv| rv["tag_name"].as_str().unwrap().len() > 0)
        .map(|v| Version::from_raw_value(v))
        .flatten()
        .collect();

    Ok(versions)
}
