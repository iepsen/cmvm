use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use crate::storage::Storage;
use crate::types::BoxError;

fn ensure_dir_exists(path: &Path) -> Result<(), BoxError> {
    if !path.exists() {
        fs::create_dir(path)?;
        println!("[cmvm] Creating {}", path.display());
    }
    Ok(())
}

pub fn bootstrap(storage: &impl Storage) -> Result<(), BoxError> {
    ensure_dir_exists(storage.get_data_dir()?.as_path())?;
    ensure_dir_exists(storage.get_versions_dir()?.as_path())?;
    ensure_dir_exists(storage.get_cache_dir()?.as_path())?;
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
