pub mod vtable;

use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::fs::read_dir;
use std::io;
use std::path::{Path, PathBuf};

use libc::c_void;
use libloading::Library;
use log;

use vtable::VTable;

#[derive(Debug)]
pub struct InitializationError;

impl fmt::Display for InitializationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error during initialization of the PeripheralManager")
    }
}

impl Error for InitializationError {
    fn description(&self) -> &str {
        "Failed to initialze the PeripheralManager"
    }
}

/// A PeripheralManager maintains the set of peripherals and their libraries.
///
/// The interface to the peripheral is a dynamically loaded library and a C API.
pub struct PeripheralManager {
    libraries: Vec<Library>,
    peripherals: Vec<*mut c_void>,
    vtables: Vec<VTable>,
}

impl PeripheralManager {
    /// Creates a new instance of a Peripheral Manager.
    pub fn new() -> PeripheralManager {
        PeripheralManager {
            libraries: Vec::new(),
            peripherals: Vec::new(),
            vtables: Vec::new(),
        }
    }

    /// Initializes the daemon process by loading peripherals.
    ///
    /// # Arguments
    ///
    /// * `dir` - A path to a directory to search for peripheral library files.
    pub fn init(&mut self, dir: &Path) -> Result<(), InitializationError> {
        let libraries = PeripheralManager::find_peripherals(&dir)
            .map_err(|e| {
                log::error!("Failed to load peripheral directory {:?}: {}", dir, e);
                InitializationError
            })?
            .ok_or_else(|| {
                log::error!("Could not load any libraries from {:?}", dir);
                InitializationError
            })?;

        self.load_peripherals(libraries);
        Ok(())
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
    /// * `libs` - A vector of `PathBuf`s pointing to library files to load.
    fn load_peripherals(&mut self, libs: Vec<PathBuf>) {
        log::debug!("Loading peripherals...");

        for lib in libs {
            let lib_str = lib
                .to_str()
                .expect("Could not convert library name to string.");

            log::info!("Attempting to load library from file: {}", lib_str);
            let lib = match Library::new(&lib) {
                Ok(lib) => {
                    log::info!("Succeeded to load library {}", lib_str);
                    lib
                }
                Err(_) => {
                    log::error!("Failed to load library {}", lib_str);
                    continue;
                }
            };

            unsafe {
                let vtable = match VTable::new(&lib) {
                    Ok(vtable) => {
                        log::info!("Succeeded to load symbols from library {}", lib_str);
                        vtable
                    }
                    Err(_) => {
                        log::error!("Failed to load vtable symbols from library {}", lib_str);
                        continue;
                    }
                };

                let peripheral: *mut c_void = (vtable.peripheral_new)();

                // Push everything at the end so that the PeripheralManager field vectors have the
                // same length.
                self.peripherals.push(peripheral);
                self.vtables.push(vtable);
            }

            log::info!("Finished loading library and symbols: {}", lib_str);
            self.libraries.push(lib);
        }
    }
}

impl Drop for PeripheralManager {
    fn drop(&mut self) {
        if !self.peripherals.is_empty() || !self.libraries.is_empty() {
            log::debug!("Unloading peripherals...");

            for (peripheral, vtable) in self.peripherals.drain(..).zip(self.vtables.drain(..)) {
                log::debug!("Unloading peripheral...");
                (vtable.peripheral_free)(peripheral);
            }

            for lib in self.libraries.drain(..) {
                drop(lib);
            }
        }
    }
}

/// The implementation of Send and Sync on the Peripheral Manager necessarily means that data
/// belonging to peripheral libraries should be thread safe.
///
/// Currently, PeripheralManager is thread safe because the methods that belong to the Peripheral
/// trait are thread safe by convention.
unsafe impl Send for PeripheralManager {}
unsafe impl Sync for PeripheralManager {}

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

        let result = PeripheralManager::find_peripherals(dir.path())
            .expect("Call to find_peripherals resulted in an error.");
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

        let result = PeripheralManager::find_peripherals(dir.path())
            .expect("Call to find_peripherals resulted in an error.");
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

        let result = PeripheralManager::find_peripherals(dir.path())
            .expect("Call to find_peripherals resulted in an error.");
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

        let mut manager = PeripheralManager::new();
        manager.load_peripherals(libs);

        assert!(!manager.libraries.is_empty());
        assert!(!manager.peripherals.is_empty());
    }

    /// load_peripherals does not return library files that do not exist.
    #[test]
    fn load_peripherals_handles_missing_library_files() {
        set_up();

        let mut lib = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        lib.push("target/debug/examples/fake_library.so");

        let mut libs: Vec<PathBuf> = Vec::new();
        libs.push(lib);

        let mut manager = PeripheralManager::new();
        manager.load_peripherals(libs);

        assert!(manager.libraries.is_empty());
        assert!(manager.peripherals.is_empty());
    }
}
