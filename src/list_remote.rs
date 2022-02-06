use std::thread;
use crate::cache;
use crate::version::{Version};
use crate::version;

pub fn get_versions() -> Result<(), Box<dyn std::error::Error>> {

  if !cache::get_file_path().exists() {
    println!("[cmvm] Fetching versions at first time...");
    if cache::generate_cache(None).is_err() {
      println!("[cmvm] Unable to generate cache");
    }
  } else {
    thread::spawn(|| {
      if cache::generate_cache(None).is_err() {
        println!("[cmvm] Unable to generate cache");
      }
    });
  }

  let contents = cache::get_cache();
  if contents.is_err() {
    println!("[cmvm] Unable to restore cache");
  }

  let versions = version::get(contents.unwrap());

  print_versions(versions);

  Ok(())
}

fn print_versions(versions: Vec<Version>) {
  println!("[cmvm] Available cmake versions:");
  for version in versions {
    println!("    {}", version.tag_name);
  }
}
