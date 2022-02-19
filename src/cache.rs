use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::constants::{CACHE_DIR, ROOT_DIR, VERSIONS_DIR};

pub fn bootstrap() -> Result<(), Box<dyn std::error::Error>> {
    if !ROOT_DIR.exists() {
        fs::create_dir(ROOT_DIR.as_path())?;
        println!("[cmvm] Creating {}", ROOT_DIR.display());
    }

    if !VERSIONS_DIR.exists() {
        fs::create_dir(VERSIONS_DIR.as_path())?;
        println!("[cmvm] Creating {}", VERSIONS_DIR.display());
    }

    if !CACHE_DIR.exists() {
        fs::create_dir(CACHE_DIR.as_path())?;
        println!("[cmvm] Creating {}", CACHE_DIR.display());
    }

    Ok(())
}

pub fn create_dir(path: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
    let destination_path = ROOT_DIR.join(path.unwrap_or(&Path::new("")));
    fs::create_dir(destination_path)?;
    Ok(())
}

pub fn create_file(path: &Path) -> Result<fs::File, Box<dyn std::error::Error>> {
    if ROOT_DIR.join(path).exists() {
        delete(Some(&ROOT_DIR.join(path)))?;
    }

    Ok(fs::File::create(ROOT_DIR.join(path))?)
}

pub fn open_file(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let mut cache_file = fs::File::options().read(true).open(path)?;
    let mut contents = String::new();
    cache_file.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn delete(path: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
    let destination_path = ROOT_DIR.join(path.unwrap_or(&Path::new("")));

    if destination_path.is_dir() {
        fs::remove_dir_all(destination_path)?;
    } else {
        fs::remove_file(destination_path)?;
    }

    Ok(())
}

pub fn ls(path: Option<&Path>) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let destination_path = ROOT_DIR.join(path.unwrap_or(&Path::new("")));
    let contents: Vec<PathBuf> = destination_path
        .read_dir()?
        .filter_map(|entry| entry.ok())
        .map(|content| content.path())
        .collect();

    Ok(contents)
}
