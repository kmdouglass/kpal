use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::fs::read_dir;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use libc::c_int;
use libloading::Library as Dll;
use libloading::Symbol;
use log;

use kpal_peripheral::constants::*;

use crate::models::Library;
use crate::plugins::TSLibrary;

/// Initializes the process of finding and loading peripheral libraries.
///
/// # Arguments
///
/// * `dir` - A path to a directory to search for peripheral library files.
pub fn init(dir: &Path) -> Result<Vec<TSLibrary>, LibraryInitError> {
    log::info!(
        "Searching for peripheral library files inside the following directory: {:?}",
        dir
    );

    let libraries = find_peripherals(&dir)
        .map_err(|e| {
            log::error!(
                "Failed to load peripheral library directory {:?}: {}",
                dir,
                e
            );
            LibraryInitError
        })?
        .ok_or_else(|| {
            log::error!("Could not load any libraries from {:?}", dir);
            LibraryInitError
        })?;

    load_peripherals(libraries).ok_or_else(|| LibraryInitError)
}

/// Finds all peripheral library files inside a directory.
///
/// # Arguments
///
/// * `dir` - A path to a directory to search for peripheral library files.
fn find_peripherals(dir: &Path) -> Result<Option<Vec<PathBuf>>, io::Error> {
    let mut peripherals: Vec<PathBuf> = Vec::new();
    log::debug!("Beginning search for peripheral libraries in {:?}", dir);
    for entry in read_dir(dir)? {
        log::debug!("Examining entry");
        let entry = entry?;
        let path = entry.path();
        log::debug!("Found candidate library file {:?}", path);

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
/// * `lib_paths` - A vector of `PathBuf`s pointing to library files to load.
fn load_peripherals(lib_paths: Vec<PathBuf>) -> Option<Vec<TSLibrary>> {
    log::debug!("Loading peripherals...");
    let (mut libraries, mut counter) = (Vec::new(), 0usize);

    for lib in lib_paths {
        let path = lib.to_str().unwrap_or("Unknown library path");

        let file_name = lib
            .file_name()
            .unwrap_or(OsStr::new("Unknown"))
            .to_string_lossy()
            .into_owned();

        log::info!("Attempting to load library from file: {}", path);
        let lib = match Dll::new(&lib) {
            Ok(lib) => {
                log::info!("Succeeded to load library {}", path);
                lib
            }
            Err(_) => {
                log::error!("Failed to load library {}", path);
                continue;
            }
        };

        log::info!("Calling initialization routine for {}", path);
        let result = match initialize_peripheral(&lib) {
            Ok(result) => result,
            Err(_) => {
                log::error!("Failed to call initialization routine for {}", path);
                continue;
            }
        };

        if result != LIBRARY_OK {
            log::error!("Initialization of {} failed: {}", path, result);
            continue;
        }

        libraries.push(Arc::new(Mutex::new(Library::new(
            counter,
            file_name,
            Some(lib),
        ))));

        counter += 1;
        log::info!("Initialization of {} succeeded.", path);
    }

    if libraries.len() != 0 {
        Some(libraries)
    } else {
        None
    }
}

fn initialize_peripheral(lib: &Dll) -> Result<c_int, io::Error> {
    unsafe {
        let init: Symbol<extern "C" fn() -> c_int> = lib.get(b"library_init\0")?;
        Ok(init())
    }
}

#[derive(Debug)]
pub struct LibraryInitError;

impl fmt::Display for LibraryInitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Library initialization error")
    }
}

impl Error for LibraryInitError {
    fn description(&self) -> &str {
        "Failed to initialze the peripheral libraries"
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

        assert!(load_peripherals(libs).is_some());
    }

    /// load_peripherals does not return library files that do not exist.
    #[test]
    fn load_peripherals_handles_missing_library_files() {
        set_up();

        let mut lib = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        lib.push("target/debug/examples/fake_library.so");

        let mut libs: Vec<PathBuf> = Vec::new();
        libs.push(lib);

        assert!(load_peripherals(libs).is_none());
    }
}