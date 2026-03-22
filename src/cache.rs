use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::storage::Storage;

fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir(path)?;
        println!("[cmvm] Creating {}", path.display());
    }
    Ok(())
}

pub fn bootstrap(storage: &impl Storage) -> Result<()> {
    ensure_dir_exists(storage.get_data_dir()?.as_path())?;
    ensure_dir_exists(storage.get_versions_dir()?.as_path())?;
    ensure_dir_exists(storage.get_cache_dir()?.as_path())?;
    Ok(())
}

pub fn create_dir(path: &Path) -> Result<()> {
    fs::create_dir(path)?;
    Ok(())
}

pub fn create_file(path: &Path, data_dir: &Path) -> Result<fs::File> {
    if data_dir.join(path).exists() {
        delete(&data_dir.join(path))?;
    }
    Ok(fs::File::create(data_dir.join(path))?)
}

pub fn open_file(path: PathBuf) -> Result<String> {
    let mut cache_file = fs::File::options().read(true).open(path)?;
    let mut contents = String::new();
    cache_file.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn delete(path: &Path) -> Result<()> {
    match path.is_dir() {
        true => fs::remove_dir_all(path)?,
        false => fs::remove_file(path)?,
    }
    Ok(())
}

pub fn ls(path: &Path) -> Result<Vec<PathBuf>> {
    let contents: Vec<PathBuf> = path
        .read_dir()?
        .filter_map(|entry| entry.ok())
        .map(|content| content.path())
        .collect();

    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::io::Write;

    #[test]
    fn test_create_dir() {
        let base = env::temp_dir().join("cmvm_test_cache_create_dir");
        let _ = fs::remove_dir_all(&base);
        create_dir(&base).unwrap();
        assert!(base.exists());
        assert!(base.is_dir());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_open_file_reads_contents() {
        let base = env::temp_dir().join("cmvm_test_cache_open_file");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let file_path = base.join("test.txt");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"hello cmvm").unwrap();

        let contents = open_file(file_path).unwrap();
        assert_eq!(contents, "hello cmvm");
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_delete_file() {
        let base = env::temp_dir().join("cmvm_test_cache_delete_file");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let file_path = base.join("to_delete.txt");
        fs::File::create(&file_path).unwrap();
        assert!(file_path.exists());

        delete(&file_path).unwrap();
        assert!(!file_path.exists());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_delete_dir() {
        let base = env::temp_dir().join("cmvm_test_cache_delete_dir");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        assert!(base.exists());

        delete(&base).unwrap();
        assert!(!base.exists());
    }

    #[test]
    fn test_ls_returns_directory_entries() {
        let base = env::temp_dir().join("cmvm_test_cache_ls");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(base.join("3.20.0")).unwrap();
        fs::create_dir_all(base.join("3.21.0")).unwrap();

        let mut entries = ls(&base).unwrap();
        entries.sort();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], base.join("3.20.0"));
        assert_eq!(entries[1], base.join("3.21.0"));
        let _ = fs::remove_dir_all(&base);
    }
}
