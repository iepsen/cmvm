use serde_json::{Value};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Asset {
  pub content_type: String,
  pub browser_download_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Version {
  pub tag_name: String,
  pub assets: Vec<Asset>
}

pub fn get(contents: String) -> Vec<Version> {
  let raw_versions: Vec<Value> = serde_json::from_str(contents.as_str()).unwrap();
  let mut versions: Vec<Version> = Vec::new();
  for raw_version in raw_versions {
    if raw_version["tag_name"].as_str().unwrap().len() > 0 {
      let version: Version = serde_json::from_value(raw_version).unwrap();
      versions.push(version);
    }
  }
  return versions;
}
