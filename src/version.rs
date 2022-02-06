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