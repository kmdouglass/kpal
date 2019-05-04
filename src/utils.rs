use std::ffi::OsStr;
use std::fs::read_dir;
use std::io::Error;
use std::path::{Path, PathBuf};

use libloading::Library;
use log;

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

/// Loads a list of plugin library files.
///
/// # Arguments
///
/// * `libs` - A vector of `PathBuf`s pointing to library files to load.
pub fn load_plugins(libs: Vec<PathBuf>) -> Option<Vec<Library>> {
    let mut plugins: Vec<Library> = Vec::new();
    for lib in libs {
        log::info!("Attempting to load library from file: {:?}", lib);
        let lib = match Library::new(&lib) {
            Ok(lib) => {
                log::info!("Succeeded to load library from file: {:?}", lib);
                lib
            }
            Err(_) => {
                log::error!("Failed to load library from file: {:?}", lib);
                continue;
            }
        };

        plugins.push(lib);
    }

    if plugins.len() != 0 {
        Some(plugins)
    } else {
        None
    }
}
