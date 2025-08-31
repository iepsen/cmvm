use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use crate::storage::Storage;
use crate::types::BoxError;

pub fn bootstrap(storage: &impl Storage) -> Result<(), BoxError> {
    let data_dir = storage.get_data_dir()?;
    let cache_dir = storage.get_cache_dir()?;
    let versions_dir = storage.get_versions_dir()?;
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

pub fn create_dir(path: &Path) -> Result<(), BoxError> {
    fs::create_dir(path)?;
    Ok(())
}

pub fn create_file(path: &Path, data_dir: &Path) -> Result<fs::File, BoxError> {
    if data_dir.join(path).exists() {
        delete(&data_dir.join(path))?;
    }
    Ok(fs::File::create(data_dir.join(path))?)
}

pub fn open_file(path: PathBuf) -> Result<String, BoxError> {
    let mut cache_file = fs::File::options().read(true).open(path)?;
    let mut contents = String::new();
    cache_file.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn delete(path: &Path) -> Result<(), BoxError> {
    match path.is_dir() {
        true => fs::remove_dir_all(path)?,
        false => fs::remove_file(path)?,
    }

    Ok(())
}

pub fn ls(path: &Path) -> Result<Vec<PathBuf>, BoxError> {
    let contents: Vec<PathBuf> = path
        .read_dir()?
        .filter_map(|entry| entry.ok())
        .map(|content| content.path())
        .collect();

    Ok(contents)
}
