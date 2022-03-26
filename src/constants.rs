use lazy_static::lazy_static;

lazy_static! {
  #[derive(Debug)]
  pub static ref SUPPORTED_PLATFORMS: Vec<String> = {
    vec!["macos".to_string(), "linux".to_string()]
  };
}

pub static BASE_URL: &str = "https://api.github.com/repos/Kitware/CMake/releases";

pub static RELEASES_FILE_NAME: &str = "releases.json";
