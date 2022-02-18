use crate::{package, releases, versions};

pub fn install_version(v: &str) -> Result<(), Box<dyn std::error::Error>> {
  releases::build_cache()?;

  if let Some(version) = releases::get_release(v.trim().to_string()) {
    match package::get_cmake_release(&version) {
      Ok(()) => {
        println!("[cmvm] Version {} installed successfully.", version.tag_name);
        use_version(&v)?;
        println!("[cmvm] Done.");
      },
      Err(e) => println!("[cmvm] Error while installing version {}: {}", version.tag_name, e),
    }
  } else {
    println!("[cmvm] Version {} not found.", v);
  }
  Ok(())
}

pub fn list_remote_versions() -> Result<(), Box<dyn std::error::Error>> {
  releases::build_cache()?;
  
  println!("[cmvm] List of available versions to install:");
  match versions::list_remote_versions() {
    Ok(versions) => println!("{}", versions),
    Err(_) => println!("[cmvm] There is no versions installed yet."),
  };
  Ok(())
}

pub fn list_versions() -> Result<(), Box<dyn std::error::Error>> {
  match versions::list_versions() {
    Ok(versions) => {
      if versions.len() > 0 {
        println!("[cmvm] Installed versions:");
        println!("{}", versions);
      } else {
        println!("[cmvm] No cmake versions installed yet. Use `cmvm install <version>` to install your first cmake version.");
        println!("[cmvm] Type `cmvm help` for more information.");    
      }
    },
    Err(_) => println!("[cmvm] There is no versions installed yet."),
  };
  Ok(())
}

pub fn use_version(v: &str) -> Result<(), Box<dyn std::error::Error>> {
  if let Some(version) = releases::get_release(v.trim().to_string()) {
    match versions::use_version(&version) {
      Ok(_) => println!("[cmvm] Version {} set as default.", version.tag_name),
      Err(e) => println!("[cmvm] Error when trying to set version {}: {}", version.tag_name, e),
    }
  } else {
    println!("[cmvm] Version {} not found.", v);
  }
  Ok(())
}
