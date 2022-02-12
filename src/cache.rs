use std::{fs::{create_dir, remove_dir_all, remove_file, File}, path::{Path, PathBuf}, io::Read};
use crate::constants::{ROOT_DIR};

pub fn write_dir(path: Option<&Path>) -> std::io::Result<()> {
  let destination_path = ROOT_DIR.join(path.unwrap_or(&Path::new("")));
  create_dir(destination_path)?;
  Ok(())
}

pub fn write_file(path: &Path) -> std::io::Result<File> {
  return File::create(ROOT_DIR.join(path));
}

pub fn open_file(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
  let mut cache_file = File::options().read(true).open(path)?;
  let mut contents = String::new();
  
  cache_file.read_to_string(&mut contents)?;
  return Ok(contents);
}

pub fn delete(path: Option<&Path>) -> std::io::Result<()> {
  let destination_path = ROOT_DIR.join(path.unwrap_or(&Path::new("")));

  if destination_path.is_dir() {
    remove_dir_all(destination_path)?;
  } else {
    remove_file(destination_path)?;
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