use std::fs;
use std::path::{Path, PathBuf};
use std::io::Read;

use crate::constants::{ROOT_DIR, CACHE_DIR, VERSIONS_DIR};

pub fn bootstrap() {
    if !ROOT_DIR.exists() {
    if fs::create_dir(ROOT_DIR.as_path()).is_err() {
      println!("[cmvm] Unable to create .cmvm dir");
      return;
    }
  }

  if !VERSIONS_DIR.exists() {
    if fs::create_dir(VERSIONS_DIR.as_path()).is_err() {
      println!("[cmvm] Unable to create .cmvm/versions dir");
      return;
    }
  }
  
  if !CACHE_DIR.exists() {
    if fs::create_dir(CACHE_DIR.as_path()).is_err() {
      println!("[cmvm] Unable to create .cmvm/cache dir");
      return;
    }
  }
}

pub fn create_dir(path: Option<&Path>) -> std::io::Result<()> {
  let destination_path = ROOT_DIR.join(path.unwrap_or(&Path::new("")));
  fs::create_dir(destination_path)?;
  Ok(())
}

pub fn create_file(path: &Path) -> Result<fs::File, std::io::Error> {
  if ROOT_DIR.join(path).exists() {
    delete(Some(&ROOT_DIR.join(path))).unwrap();
  }
  
  fs::File::create(ROOT_DIR.join(path))
}

pub fn open_file(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
  let mut cache_file = fs::File::options().read(true).open(path)?;
  let mut contents = String::new();
  cache_file.read_to_string(&mut contents)?;
  
  Ok(contents)
}

pub fn delete(path: Option<&Path>) -> std::io::Result<()> {
  let destination_path = ROOT_DIR.join(path.unwrap_or(&Path::new("")));

  if destination_path.is_dir() {
    fs::remove_dir_all(destination_path)?;
  } else {
    fs::remove_file(destination_path)?;
  }

  Ok(())
}

pub fn ls(path: Option<&Path>) -> std::io::Result<Vec<PathBuf>> {
  let destination_path = ROOT_DIR.join(path.unwrap_or(&Path::new("")));
  let contents: Vec<PathBuf> = destination_path
    .read_dir()
    .unwrap()
    .filter_map(|entry| entry.ok())
    .map(|content| content.path())
    .collect();

  Ok(contents)
}