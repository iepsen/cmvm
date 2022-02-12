use crate::versions;

pub fn install_version(version: &str) {
  println!("Version to be installed: {}", version);
}

pub fn list_remote_versions() {
  println!("[cmvm] List of available versions to install:");
  match versions::list_remote_versions() {
    Err(_) => println!("[cmvm] There is no versions installed yet."),
    Ok(versions) => println!("{}", versions),
  };
}

pub fn list_versions() {
  println!("[cmvm] Installed versions:");
  match versions::list_versions() {
    Err(_) => println!("[cmvm] There is no versions installed yet."),
    Ok(versions) => println!("{}", versions),
  };
}

pub fn use_version(version: &str) {
  match versions::use_version(version) {
    Ok(_) => println!("[cmvm] {} set as default version.", version),
    Err(e) => println!("[cmvm] Error when trying to set version {}: {}", version, e),
  }
}
