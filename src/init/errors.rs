use std::{boxed::Box, error::Error, fmt};

use crate::init::libraries::LibraryInitError;

/// Raised when an error occurs during the daemon's initialization.
#[derive(Debug)]
pub struct InitError {
    side: Option<Box<dyn Error + 'static>>,
}

impl InitError {
    fn new(error: Option<Box<dyn Error + 'static>>) -> InitError {
        InitError { side: error }
    }
}

impl Error for InitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.side.as_ref().map(|e| e.as_ref())
    }
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InitError {{ Cause: {:?} }}", self.side)
    }
}

impl From<LibraryInitError> for InitError {
    fn from(error: LibraryInitError) -> InitError {
        InitError::new(Some(Box::new(error)))
    }
}
