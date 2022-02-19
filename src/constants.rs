use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
  #[derive(Debug)]
  pub static ref ROOT_DIR: PathBuf = {
    dirs::home_dir().unwrap().join(".cmvm")
  };
  #[derive(Debug)]
  pub static ref CACHE_DIR: PathBuf = {
    ROOT_DIR.join("cache")
  };
  #[derive(Debug)]
  pub static ref VERSIONS_DIR: PathBuf = {
    ROOT_DIR.join("versions")
  };

  #[derive(Debug)]
  pub static ref CURRENT_VERSION: PathBuf = {
    ROOT_DIR.join("current")
  };

}

pub static BASE_URL: &str = "https://api.github.com/repos/Kitware/CMake/releases";

pub static BASE_RELEASE_URL: &str = "https://github.com/Kitware/CMake/releases/download";

pub static RELEASES_FILE_NAME: &str = "releases.json";
