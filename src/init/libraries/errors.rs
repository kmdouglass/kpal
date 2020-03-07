use std::{boxed::Box, error::Error, fmt, io};

use crate::plugins::PluginError;

/// A general error that is raised while initializing plugin libraries.
#[derive(Debug)]
pub struct LibraryInitError {
    side: Option<Box<dyn Error + 'static>>,
}

impl LibraryInitError {
    pub fn new(error: Option<Box<dyn Error + 'static>>) -> LibraryInitError {
        LibraryInitError { side: error }
    }
}

impl Error for LibraryInitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.side.as_ref().map(|e| e.as_ref())
    }
}

impl fmt::Display for LibraryInitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LibraryInitError {{ Cause: {:?} }}", self.side)
    }
}

impl From<io::Error> for LibraryInitError {
    fn from(error: io::Error) -> LibraryInitError {
        LibraryInitError::new(Some(Box::new(error)))
    }
}

impl From<NoLibrariesFoundError> for LibraryInitError {
    fn from(error: NoLibrariesFoundError) -> LibraryInitError {
        LibraryInitError::new(Some(Box::new(error)))
    }
}

impl From<NoLibrariesLoadedError> for LibraryInitError {
    fn from(error: NoLibrariesLoadedError) -> LibraryInitError {
        LibraryInitError::new(Some(Box::new(error)))
    }
}

impl From<PluginError> for LibraryInitError {
    fn from(error: PluginError) -> LibraryInitError {
        LibraryInitError::new(Some(Box::new(error)))
    }
}

/// An error that is raised when no libraries could found.
#[derive(Debug)]
pub struct NoLibrariesFoundError {}

impl Error for NoLibrariesFoundError {}

impl fmt::Display for NoLibrariesFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not find any plugin libraries")
    }
}

/// An error that is raised when no libraries could loaded.
#[derive(Debug)]
pub struct NoLibrariesLoadedError {}

impl Error for NoLibrariesLoadedError {}

impl fmt::Display for NoLibrariesLoadedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not load any plugin libraries")
    }
}
