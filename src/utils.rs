use std::ffi::OsStr;
use std::fs::read_dir;
use std::io::Error;
use std::path::{Path, PathBuf};

/// Finds all plugin library files inside a directory.
///
/// # Arguments
///
/// * `dir` - A path to a directory to search for plugin library files.
pub fn find_plugins(dir: &Path) -> Result<Option<Vec<PathBuf>>, Error> {
    let mut plugins: Vec<PathBuf> = Vec::new();
    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let extension: &OsStr = match path.extension() {
                Some(ext) => ext,
                None => continue,
            };

            if extension == "so" {
                plugins.push(path);
            }
        }
    }

    if plugins.len() != 0 {
        Ok(Some(plugins))
    } else {
        Ok(None)
    }
}
