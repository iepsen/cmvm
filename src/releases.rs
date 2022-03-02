use crate::cache;
use crate::constants::{BASE_URL, CACHE_DIR, CURRENT_VERSION, RELEASES_FILE_NAME, VERSIONS_DIR};
use crate::http;
use crate::versions::Version;
use serde_json::Value;
use std::{fs, io::Write, thread};

pub fn build_cache() -> Result<(), Box<dyn std::error::Error>> {
    if !CACHE_DIR.join(RELEASES_FILE_NAME).exists() {
        println!("[cmvm] Fetching versions at first time...");
        if cache_releases(None).is_err() {
            println!("[cmvm] Failed to fetch remote versions");
        }
    } else {
        thread::spawn(|| {
            if cache_releases(None).is_err() {
                println!("[cmvm] Failed to fetch remote versions");
            }
        });
    }
    Ok(())
}

pub fn get_release(version: &String) -> Result<Option<Version>, Box<dyn std::error::Error>> {
    let releases = get_releases()?;
    let release = releases.iter().find(|v| &v.get_tag_name() == version);

    if let Some(release) = release {
        let mut found_release: Version = release.clone();
        found_release.set_tag_name(release.get_tag_name());
        return Ok(Some(found_release));
    }
    Ok(None)
}

pub fn delete_cache_release(version: &String) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(release) = get_release(version)? {
        let version_path = VERSIONS_DIR.join(release.get_tag_name());
        if CURRENT_VERSION.read_link()? == version_path {
            cache::delete(Some(&CURRENT_VERSION))?;
        }
        cache::delete(Some(version_path.as_path()))?;
    }

    Ok(())
}

fn cache_releases(page: Option<i32>) -> Result<(), Box<dyn std::error::Error>> {
    let current_page = page.unwrap_or(1);
    let first_page = current_page == 1;
    let mut response = http::get(format!("{}?page={}", BASE_URL, current_page).as_str())?;

    if !response.status().is_success() {
        Err("[cmvm] Something went wrong")?;
    }

    let current_page_file = CACHE_DIR.join(format!("{}.json", current_page));

    if current_page_file.exists() {
        cache::delete(Some(&current_page_file))?;
    }

    let mut file = cache::create_file(current_page_file.as_path())?;
    response.copy_to(&mut file)?;

    if first_page {
        if let Some(link_header) = response.headers().get("link") {
            let pages = get_number_of_pages(link_header.to_str()?)?;
            for page in 2..=pages {
                cache_releases(Some(page))?;
            }
            merge(pages)?;
        }
    }
    Ok(())
}

fn merge(pages: i32) -> Result<(), Box<dyn std::error::Error>> {
    let mut releases: Vec<Value> = Vec::new();

    if CACHE_DIR.join(RELEASES_FILE_NAME).exists() {
        cache::delete(Some(&CACHE_DIR.join(RELEASES_FILE_NAME)))?;
    }

    for page in 1..=pages {
        let page_file = CACHE_DIR.join(format!("{}.json", page));

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

    let mut cache_file = cache::create_file(&CACHE_DIR.join(RELEASES_FILE_NAME))?;
    let cache_json = serde_json::to_string(&releases)?;
    cache_file.write(cache_json.as_bytes())?;

    Ok(())
}

fn get_number_of_pages(link_header: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let mut last_page = 1;
    let parsed_link_header = parse_link_header::parse(link_header)?;
    let last_link = parsed_link_header.get(&Some("last".to_string()));
    if let Some(last_link) = last_link {
        last_page = last_link.queries["page"].parse::<i32>()?;
    }
    Ok(last_page)
}

fn get_releases() -> Result<Vec<Version>, Box<dyn std::error::Error>> {
    let releases = cache::open_file(CACHE_DIR.join(RELEASES_FILE_NAME));
    let raw_versions: Vec<Value> = serde_json::from_str(releases.unwrap().as_str())?;
    let mut versions: Vec<Version> = Vec::new();
    for raw_version in raw_versions {
        if raw_version["tag_name"].as_str().unwrap().len() > 0 {
            let version: Version = serde_json::from_value(raw_version)?;
            versions.push(version);
        }
    }
    Ok(versions)
}
