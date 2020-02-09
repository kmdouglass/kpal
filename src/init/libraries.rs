//! Methods for loading and initializing plugin libraries.
use std::{
    error::Error,
    ffi::OsStr,
    fmt,
    fs::read_dir,
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use libc::c_int;
use libloading::{Library as Dll, Symbol};
use log;

use kpal_plugin::{error_codes::*, KpalLibraryInit, Plugin};

use crate::{
    models::Library,
    plugins::{kpal_plugin_new, Executor},
};

/// A thread safe version of a [Library](../models/struct.Library.html) instance.
///
/// This is a convenience type for sharing a single a Library instance between multiple
/// threads. Due to its use of a Mutex, different peripherals that use the same library will not
/// make function calls from the library in a deterministic order.
pub type TSLibrary = Arc<Mutex<Library>>;

/// Returns a list of loaded plugin libraries.
///
/// # Arguments
///
/// * `dir` - A path to a directory to search for plugin library files
pub fn init(dir: &Path) -> Result<Vec<TSLibrary>, LibraryInitError> {
    log::info!(
        "Searching for peripheral library files inside the following directory: {:?}",
        dir
    );

    let libraries = find_libraries(&dir)
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

    load_libraries(libraries).ok_or_else(|| LibraryInitError)
}

/// Finds all plugin library files inside a directory.
///
/// # Arguments
///
/// * `dir` - A path to a directory to search for plugin library files
fn find_libraries(dir: &Path) -> Result<Option<Vec<PathBuf>>, io::Error> {
    let mut peripherals: Vec<PathBuf> = Vec::new();
    log::debug!("Beginning search for peripheral libraries in {:?}", dir);
    for entry in read_dir(dir)? {
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

    if !peripherals.is_empty() {
        Ok(Some(peripherals))
    } else {
        Ok(None)
    }
}

/// Loads a list of plugin library files.
///
/// # Arguments
///
/// * `lib_paths` - A vector of `PathBuf`s pointing to library files to load
fn load_libraries(lib_paths: Vec<PathBuf>) -> Option<Vec<TSLibrary>> {
    log::debug!("Loading peripherals...");
    let (mut libraries, mut counter) = (Vec::new(), 0usize);

    for lib in lib_paths {
        let path = lib.to_str().unwrap_or("Unknown library path");

        let file_name = lib
            .file_name()
            .unwrap_or_else(|| OsStr::new("Unknown"))
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
        let result = match init_library(&lib) {
            Ok(result) => result,
            Err(_) => {
                log::error!("Failed to call initialization routine for {}", path);
                continue;
            }
        };

        if result != PLUGIN_OK {
            log::error!("Initialization of {} failed: {}", path, result);
            continue;
        }

        let mut new_lib = Library::new(counter, file_name, Some(lib));
        if init_library_attributes(&mut new_lib).is_err() {
            log::error!("Failed to initialize library attributes: {:?}", new_lib);
            continue;
        };

        libraries.push(Arc::new(Mutex::new(new_lib)));
        counter += 1;
        log::info!("Initialization of {} succeeded.", path);
    }

    if !libraries.is_empty() {
        Some(libraries)
    } else {
        None
    }
}

/// Calls the initialization callback function of the library.
///
/// The integer return code of the callback is returned in the Ok variant of the result.
///
/// # Arguments
///
/// * `lib` - The library to initialize
fn init_library(lib: &Dll) -> Result<c_int, io::Error> {
    unsafe {
        let init: Symbol<KpalLibraryInit> = lib.get(b"kpal_library_init\0")?;
        Ok(init())
    }
}

fn init_library_attributes(lib: &mut Library) -> Result<(), LibraryInitError> {
    let plugin: Plugin = unsafe { kpal_plugin_new(lib).map_err(|_| LibraryInitError {})? };
    let mut executor = Executor::new(plugin);
    let attrs = executor
        .discover_attributes()
        .ok_or_else(|| LibraryInitError {})?;
    lib.set_attributes(attrs);

    Ok(())
}

/// A general error that is raised while initializing the libraries.
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

    use std::env;
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

    /// find_libraries works when only library files are present.
    #[test]
    fn find_libraries_library_files_only() {
        set_up();

        let dir = tempdir().expect("Could not create temporary directory for test data.");
        let libs: Vec<PathBuf> =
            create_dummy_files(&dir, vec!["peripheral_1.so", "peripheral_2.so"])
                .expect("Could not create test data files");

        let result =
            find_libraries(dir.path()).expect("Call to find_libraries resulted in an error.");
        let mut found_libs = match result {
            Some(libs) => libs,
            None => panic!("Found no libraries in the test data folder."),
        };
        found_libs.sort();

        assert_eq!(libs[0], found_libs[0]);
        assert_eq!(libs[1], found_libs[1]);
        assert_eq!(libs.len(), found_libs.len());
    }

    /// find_libraries works when library files and other file types are present.
    #[test]
    fn find_libraries_mixed_file_types() {
        set_up();

        let dir = tempdir().expect("Could not create temporary directory for test data.");
        let libs: Vec<PathBuf> =
            create_dummy_files(&dir, vec!["peripheral_1.so", "peripheral_2.so", "data.txt"])
                .expect("Could not create test data files");

        let result =
            find_libraries(dir.path()).expect("Call to find_libraries resulted in an error.");
        let mut found_libs = match result {
            Some(libs) => libs,
            None => panic!("Found no libraries in the test data folder."),
        };
        found_libs.sort();

        assert_eq!(libs[0], found_libs[0]);
        assert_eq!(libs[1], found_libs[1]);
        assert_eq!(2, found_libs.len());
    }

    /// find_libraries returns None when no library files are present.
    #[test]
    fn find_libraries_no_peripheral_library_files() {
        set_up();

        let dir = tempdir().expect("Could not create temporary directory for test data.");
        create_dummy_files(&dir, vec!["data.txt"]).expect("Could not create test data files");

        let result =
            find_libraries(dir.path()).expect("Call to find_libraries resulted in an error.");
        assert_eq!(None, result);
    }

    /// load_libraries works for a list of correct library files.
    #[test]
    fn load_libraries_loads_library_files() {
        set_up();

        let lib = {
            let mut dir = env::current_exe().expect("Could not determine current executable");
            dir.pop(); // Drop executable name
            dir.pop(); // Move up one directory from deps
            dir.push("examples/libbasic-plugin.so");
            dir
        };

        let mut libs: Vec<PathBuf> = Vec::new();
        libs.push(lib);

        assert!(load_libraries(libs).is_some());
    }

    /// load_libraries does not return library files that do not exist.
    #[test]
    fn load_libraries_handles_missing_library_files() {
        set_up();

        let mut lib = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        lib.push("target/debug/examples/fake_library.so");

        let mut libs: Vec<PathBuf> = Vec::new();
        libs.push(lib);

        assert!(load_libraries(libs).is_none());
    }
}
