use crate::cache;
use crate::version::{Version};
use crate::version;

pub fn version(install_version: &str) {
  let contents = cache::get_cache();
  if contents.is_err() {
    println!("[cmvm] Unable to restore cache");
  }
  let versions = version::get(contents.unwrap());

  for version in versions {
    if version.tag_name == install_version {
      download(version);
      break;
    }
  }
}

fn download(version: Version) {
  for asset in version.assets {
    if asset.content_type == "application/json" {
      let platform = "macos10.10".to_string();
      let _ = cache::cmake_version(
        version.tag_name.as_str(),
        asset.browser_download_url.as_str(),
        &platform
      );
      break;
    }
  }
}
