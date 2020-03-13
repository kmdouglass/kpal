use std::{
    boxed::Box,
    error::Error,
    ffi::{IntoStringError, NulError},
    fmt,
};

/// An error raised when schema conversions and/or validations fail.
#[derive(Debug)]
pub struct SchemaError {
    /// The cause of the error, if any.
    side: Option<Box<dyn Error + 'static>>,
}

impl Error for SchemaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.side.as_ref().map(|e| e.as_ref())
    }
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SchemaError {{ Cause: {:?} }}", self.side)
    }
}

impl From<IntoStringError> for SchemaError {
    fn from(error: IntoStringError) -> SchemaError {
        SchemaError {
            side: Some(Box::new(error)),
        }
    }
}

impl From<NulError> for SchemaError {
    fn from(error: NulError) -> SchemaError {
        SchemaError {
            side: Some(Box::new(error)),
        }
    }
}
