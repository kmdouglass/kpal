//! Error types for the Models module.

use std::{error::Error, ffi::FromBytesWithNulError, ffi::NulError, fmt, str::Utf8Error};

/// An error returned when a manipulation on a Model fails.
#[derive(Debug)]
pub enum ModelError {
    BuilderNotInitializedError,
    CannotCreateCStr(FromBytesWithNulError),
    CannotCreateCString(NulError),
    CannotCreateString(Utf8Error),
}

impl Error for ModelError {}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ModelError::*;

        match self {
            BuilderNotInitializedError => write!(f, "builder is not yet fully initialized"),
            CannotCreateCStr(e) => write!(f, "cannot create string Attribute because there is an interior nul byte in the input\nCaused by: {}", e),
            CannotCreateCString(e) => write!(f, "cannot create new CString because there is an interior nul byte in the input\nCaused by: {}", e),
            CannotCreateString(e) => write!(f, "cannot create Attribute because string is not valid UTF8\nCaused by: {}", e),
        }
    }
}

impl From<FromBytesWithNulError> for ModelError {
    fn from(error: FromBytesWithNulError) -> Self {
        ModelError::CannotCreateCStr(error)
    }
}

impl From<NulError> for ModelError {
    fn from(error: NulError) -> Self {
        ModelError::CannotCreateCString(error)
    }
}

impl From<Utf8Error> for ModelError {
    fn from(error: Utf8Error) -> Self {
        ModelError::CannotCreateString(error)
    }
}
