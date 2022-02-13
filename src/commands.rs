use crate::{package, releases, versions};

pub fn install_version(v: &str) {
  releases::build_cache();

  if let Some(version) = releases::get_release(v.to_string()) {
    match package::get_cmake_release(&version) {
      Ok(()) => {
        println!("[cmvm] Version {} installed successfully.", version.tag_name);
        use_version(&v);
      },
      Err(e) => println!("[cmvm] Error while installing version {}: {}", version.tag_name, e),
    }
  } else {
    println!("[cmvm] Version {} not found.", v);
  }
}

pub fn list_remote_versions() {
  releases::build_cache();
  
  println!("[cmvm] List of available versions to install:");
  match versions::list_remote_versions() {
    Ok(versions) => println!("{}", versions),
    Err(_) => println!("[cmvm] There is no versions installed yet."),
  };
}

pub fn list_versions() {
  println!("[cmvm] Installed versions:");
  match versions::list_versions() {
    Ok(versions) => println!("{}", versions),
    Err(_) => println!("[cmvm] There is no versions installed yet."),
  };
}

pub fn use_version(v: &str) {
  if let Some(version) = releases::get_release(v.to_string()) {
    match versions::use_version(&version) {
      Ok(_) => println!("[cmvm] {} set as default version.", version.tag_name),
      Err(e) => println!("[cmvm] Error when trying to set version {}: {}", version.tag_name, e),
    }
  } else {
    println!("[cmvm] Version {} not found.", v);
  }
}
