use std::{boxed::Box, error::Error, ffi::FromBytesWithNulError, fmt, str::Utf8Error};

/// An error type that represents a failure to convert a Val to a Value.
#[derive(Debug)]
pub struct AttributeError {
    side: ValueConversionError,
}

impl Error for AttributeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.side)
    }
}

impl fmt::Display for AttributeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AttributeError: {:?}", self)
    }
}

impl From<ValueConversionError> for AttributeError {
    fn from(error: ValueConversionError) -> Self {
        AttributeError { side: error }
    }
}

/// An error type that represents a failure to convert a Val to a Value.
#[derive(Debug)]
pub struct ValueConversionError {
    side: Box<dyn Error>,
}

impl Error for ValueConversionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.side)
    }
}

impl fmt::Display for ValueConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ValueConversionError: {:?}", self)
    }
}

impl From<FromBytesWithNulError> for ValueConversionError {
    fn from(error: FromBytesWithNulError) -> Self {
        ValueConversionError {
            side: Box::new(error),
        }
    }
}

impl From<Utf8Error> for ValueConversionError {
    fn from(error: Utf8Error) -> Self {
        ValueConversionError {
            side: Box::new(error),
        }
    }
}
