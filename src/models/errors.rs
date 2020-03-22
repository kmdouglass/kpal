//! Error types for the Models module.

use std::{
    boxed::Box, error::Error, ffi::FromBytesWithNulError, ffi::NulError, fmt, str::Utf8Error,
};

/// An error returned when a manipulation on a Model fails.
#[derive(Debug)]
pub struct ModelError {
    side: Option<Box<dyn Error + 'static + Send>>,
}

impl Error for ModelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // The `as &_` is necessary for successful type inference due to the Send trait.
        self.side.as_ref().map(|e| e.as_ref() as &_)
    }
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ModelError: {{ Cause: {:?} }}", self.side)
    }
}

impl From<BuilderPartiallyInitializedError> for ModelError {
    fn from(error: BuilderPartiallyInitializedError) -> Self {
        ModelError {
            side: Some(Box::new(error)),
        }
    }
}

impl From<FromBytesWithNulError> for ModelError {
    fn from(error: FromBytesWithNulError) -> Self {
        ModelError {
            side: Some(Box::new(error)),
        }
    }
}

impl From<NulError> for ModelError {
    fn from(error: NulError) -> Self {
        ModelError {
            side: Some(Box::new(error)),
        }
    }
}

impl From<Utf8Error> for ModelError {
    fn from(error: Utf8Error) -> Self {
        ModelError {
            side: Some(Box::new(error)),
        }
    }
}

/// An error raised when trying to build a model from a partially-initialized builder.
#[derive(Debug)]
pub struct BuilderPartiallyInitializedError();

impl Error for BuilderPartiallyInitializedError {}

impl fmt::Display for BuilderPartiallyInitializedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
