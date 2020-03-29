//! Structures that provide error information to clients of the plugin library.

use std::{ffi::FromBytesWithNulError, fmt};

/// The Error type returned by calls to the kpal-plugin library.
#[derive(Debug)]
pub enum Error {
    /// Raised when a plugin is assumed to be in its run phase but it has not yet been initialized.
    PluginUninitialized,

    /// Raised when a Val cannot be converted to a Value.
    ValueConversionError(FromBytesWithNulError),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::PluginUninitialized => None,
            Error::ValueConversionError(e) => Some(e),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::PluginUninitialized => write!(f, "Plugin is not yet initialized"),
            Error::ValueConversionError(e) => write!(f, "Value conversion error\nCaused by: {}", e),
        }
    }
}

/// An error type that represents a failure to convert a Val to a Value.
#[derive(Debug)]
pub struct ValueConversionError {
    side: FromBytesWithNulError,
}

impl From<FromBytesWithNulError> for Error {
    fn from(error: FromBytesWithNulError) -> Error {
        Error::ValueConversionError(error)
    }
}
