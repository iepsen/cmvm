use crate::Config;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use crate::config::ConfigImp;

pub fn bootstrap() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigImp::new();
    let data_dir = config.get_data_dir()?;
    let cache_dir = config.get_cache_dir()?;
    let versions_dir = config.get_versions_dir()?;
    if !data_dir.exists() {
        fs::create_dir(data_dir.as_path())?;
        println!("[cmvm] Creating {}", data_dir.display());
    }

    if !versions_dir.exists() {
        fs::create_dir(versions_dir.as_path())?;
        println!("[cmvm] Creating {}", versions_dir.display());
    }

    if !cache_dir.exists() {
        fs::create_dir(cache_dir.as_path())?;
        println!("[cmvm] Creating {}", cache_dir.display());
    }

    Ok(())
}

pub fn create_dir(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir(path)?;
    Ok(())
}

pub fn create_file(path: &Path) -> Result<fs::File, Box<dyn std::error::Error>> {
    let config = ConfigImp::new();
    let data_dir = config.get_data_dir()?;
    if data_dir.join(path).exists() {
        delete(&data_dir.join(path))?;
    }

    Ok(fs::File::create(data_dir.join(path))?)
}

pub fn open_file(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let mut cache_file = fs::File::options().read(true).open(path)?;
    let mut contents = String::new();
    cache_file.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn delete(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }

    Ok(())
}

pub fn ls(path: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let contents: Vec<PathBuf> = path
        .read_dir()?
        .filter_map(|entry| entry.ok())
        .map(|content| content.path())
        .collect();

    Ok(contents)
}
