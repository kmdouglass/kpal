use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::io::Error;
use std::path::{Path, PathBuf};

use libloading::Library;
use log;

use crate::peripherals::Peripheral;

/// Initializes the daemon process by loading peripherals.
///
/// # Arguments
///
/// * `dir` - A path to a directory to search for peripheral library files.
pub fn init(dir: &Path) -> Option<HashMap<usize, Peripheral>> {
    let libs = match find_peripherals(&dir) {
        Ok(libs) => match libs {
            Some(libs) => load_peripherals(libs),
            None => None,
        },

        Err(_) => {
            log::error!("Could not open peripheral library directory.");
            None
        }
    };

    libs.map(libs_to_hashmap)
}

/// Converts a vector of dynamic libraries into a HashMap of peripherals with integer keys.
///
/// # Arguments
///
/// * `libs` - A vector of Library objects.
fn libs_to_hashmap(libs: Vec<Library>) -> HashMap<usize, Peripheral> {
    let mut peripherals: HashMap<usize, Peripheral> = HashMap::new();
    for (id, lib) in libs.into_iter().enumerate() {
        peripherals.insert(id, Peripheral::new(lib));
    }

    peripherals
}

/// Finds all peripheral library files inside a directory.
///
/// # Arguments
///
/// * `dir` - A path to a directory to search for peripheral library files.
fn find_peripherals(dir: &Path) -> Result<Option<Vec<PathBuf>>, Error> {
    let mut peripherals: Vec<PathBuf> = Vec::new();
    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let extension: &OsStr = match path.extension() {
                Some(ext) => ext,
                None => continue,
            };

            if extension == "so" {
                peripherals.push(path);
            }
        }
    }

    if peripherals.len() != 0 {
        Ok(Some(peripherals))
    } else {
        Ok(None)
    }
}

/// Loads a list of peripheral library files.
///
/// # Arguments
///
/// * `libs` - A vector of `PathBuf`s pointing to library files to load.
fn load_peripherals(libs: Vec<PathBuf>) -> Option<Vec<Library>> {
    let mut peripherals: Vec<Library> = Vec::new();
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

        peripherals.push(lib);
    }

    if peripherals.len() != 0 {
        Some(peripherals)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::Error;
    use std::path::PathBuf;

    use env_logger;
    use tempfile::{tempdir, TempDir};

    fn set_up() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn create_dummy_files(dir: &TempDir, files: Vec<&str>) -> Result<Vec<PathBuf>, Error> {
        let path = dir.path();
        let mut libs: Vec<PathBuf> = Vec::new();
        for file in files.iter() {
            let file = path.join(file);
            File::create(&file)?;
            libs.push(file);
        }

        Ok(libs)
    }

    /// find_peripherals works when only library files are present.
    #[test]
    fn find_peripherals_library_files_only() {
        set_up();

        let dir = tempdir().expect("Could not create temporary directory for test data.");
        let libs: Vec<PathBuf> =
            create_dummy_files(&dir, vec!["peripheral_1.so", "peripheral_2.so"])
                .expect("Could not create test data files");

        let result =
            find_peripherals(dir.path()).expect("Call to find_peripherals resulted in an error.");
        let mut found_libs = match result {
            Some(libs) => libs,
            None => panic!("Found no libraries in the test data folder."),
        };
        found_libs.sort();

        assert_eq!(libs[0], found_libs[0]);
        assert_eq!(libs[1], found_libs[1]);
        assert_eq!(libs.len(), found_libs.len());
    }

    /// find_peripherals works when library files and other file types are present.
    #[test]
    fn find_peripherals_mixed_file_types() {
        set_up();

        let dir = tempdir().expect("Could not create temporary directory for test data.");
        let libs: Vec<PathBuf> =
            create_dummy_files(&dir, vec!["peripheral_1.so", "peripheral_2.so", "data.txt"])
                .expect("Could not create test data files");

        let result =
            find_peripherals(dir.path()).expect("Call to find_peripherals resulted in an error.");
        let mut found_libs = match result {
            Some(libs) => libs,
            None => panic!("Found no libraries in the test data folder."),
        };
        found_libs.sort();

        assert_eq!(libs[0], found_libs[0]);
        assert_eq!(libs[1], found_libs[1]);
        assert_eq!(2, found_libs.len());
    }

    /// find_peripherals returns None when no library files are present.
    #[test]
    fn find_peripherals_no_peripheral_library_files() {
        set_up();

        let dir = tempdir().expect("Could not create temporary directory for test data.");
        create_dummy_files(&dir, vec!["data.txt"]).expect("Could not create test data files");

        let result =
            find_peripherals(dir.path()).expect("Call to find_peripherals resulted in an error.");
        assert_eq!(None, result);
    }

    /// load_peripherals works for a list of correct library files.
    #[test]
    fn load_peripherals_loads_library_files() {
        set_up();

        let mut lib = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        lib.push("target/debug/examples/libbasic-peripheral.so");

        let mut libs: Vec<PathBuf> = Vec::new();
        libs.push(lib);

        let result = load_peripherals(libs);
        match result {
            Some(_) => (),
            None => panic!("load_peripherals failed to load any libraries."),
        }
    }

    /// load_peripherals does not return library files that do not exist.
    #[test]
    fn load_peripherals_handles_missing_library_files() {
        set_up();

        let mut lib = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        lib.push("target/debug/examples/fake_library.so");

        let mut libs: Vec<PathBuf> = Vec::new();
        libs.push(lib);

        let result = load_peripherals(libs);
        match result {
            Some(_) => panic!("load_peripherals found libraries that it should not have."),
            None => (),
        }
    }
}
