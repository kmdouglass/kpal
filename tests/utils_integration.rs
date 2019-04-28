use std::fs::File;
use std::io::Error;
use std::path::PathBuf;

use tempfile::{tempdir, TempDir};

use kpal::utils::find_plugins;

fn create_files(dir: &TempDir, files: Vec<&str>) -> Result<Vec<PathBuf>, Error> {
    let path = dir.path();
    let mut libs: Vec<PathBuf> = Vec::new();
    for file in files.iter() {
        let file = path.join(file);
        File::create(&file)?;
        libs.push(file);
    }

    Ok(libs)
}

/// find_plugins works when only library files are present.
#[test]
fn find_plugins_library_files_only() {
    let dir = tempdir().expect("Could not create temporary directory for test data.");
    let libs: Vec<PathBuf> = create_files(&dir, vec!["plugin_1.so", "plugin_2.so"])
        .expect("Could not create test data files");

    let result = find_plugins(dir.path()).expect("Call to find_plugins resulted in an error.");
    let mut found_libs = match result {
        Some(libs) => libs,
        None => panic!("Found no libraries in the test data folder."),
    };
    found_libs.sort();

    assert_eq!(libs[0], found_libs[0]);
    assert_eq!(libs[1], found_libs[1]);
    assert_eq!(libs.len(), found_libs.len());
}

/// find_plugins works when library files and other file types are present.
#[test]
fn find_plugins_mixed_file_types() {
    let dir = tempdir().expect("Could not create temporary directory for test data.");
    let libs: Vec<PathBuf> = create_files(&dir, vec!["plugin_1.so", "plugin_2.so", "data.txt"])
        .expect("Could not create test data files");

    let result = find_plugins(dir.path()).expect("Call to find_plugins resulted in an error.");
    let mut found_libs = match result {
        Some(libs) => libs,
        None => panic!("Found no libraries in the test data folder."),
    };
    found_libs.sort();

    assert_eq!(libs[0], found_libs[0]);
    assert_eq!(libs[1], found_libs[1]);
    assert_eq!(2, found_libs.len());
}

/// find_plugins returns None when no library files are present.
#[test]
fn find_plugins_no_plugin_library_files() {
    let dir = tempdir().expect("Could not create temporary directory for test data.");
    create_files(&dir, vec!["data.txt"]).expect("Could not create test data files");

    let result = find_plugins(dir.path()).expect("Call to find_plugins resulted in an error.");
    assert_eq!(None, result);
}
