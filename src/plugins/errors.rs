//! Error types for the plugins module.

use std::{
    error::Error,
    fmt,
    fmt::Debug,
    sync::{MutexGuard, PoisonError, RwLockWriteGuard},
};

use crate::models::AttributeError;
use crate::{init::transmitters::Transmitters, models::Library};

/// Contains information for clients about errors that occur while communicating with a plugin.
///
/// This error type is intended for the exclusive use by server request handlers. After an
/// operation is performed on a plugin, the result may be an error of one of many different
/// types. These errors should be converted into a PluginError. A PluginError contains the
/// information that is necessary for the server's request handler to report information back to
/// the client about why the requested operation failed.
#[derive(Debug)]
pub struct PluginError {
    /// The body of the HTTP response to return to the client.
    pub body: String,

    /// The HTTP status code that should be returned to the client.
    pub http_status_code: u16,
}

impl Error for PluginError {}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PluginError: {:?}", self)
    }
}

impl From<AttributeError> for PluginError {
    fn from(_error: AttributeError) -> Self {
        PluginError {
            body: "Could not create attribute from value".to_string(),
            http_status_code: 500,
        }
    }
}

impl From<std::io::Error> for PluginError {
    fn from(_error: std::io::Error) -> Self {
        PluginError {
            body: "Could not get symbol from shared library".to_string(),
            http_status_code: 500,
        }
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, Library>>> for PluginError {
    fn from(_error: PoisonError<MutexGuard<Library>>) -> Self {
        PluginError {
            body: "The Mutex on the library is poisoned".to_string(),
            http_status_code: 500,
        }
    }
}

impl<'a> From<PoisonError<RwLockWriteGuard<'a, Transmitters>>> for PluginError {
    fn from(_error: PoisonError<RwLockWriteGuard<Transmitters>>) -> Self {
        PluginError {
            body: "The RwLock on the transmitters collection is poisoned".to_string(),
            http_status_code: 500,
        }
    }
}

impl From<std::str::Utf8Error> for PluginError {
    fn from(_: std::str::Utf8Error) -> Self {
        PluginError {
            body: "Could not convert the plugin's error messago to a UTF8 string".to_string(),
            http_status_code: 500,
        }
    }
}

impl From<NameError> for PluginError {
    fn from(error: NameError) -> Self {
        match error {
            NameError::DoesNotExist(msg) => PluginError {
                body: msg,
                http_status_code: 404,
            },
            NameError::Failure(msg) => PluginError {
                body: msg,
                http_status_code: 500,
            },
        }
    }
}

impl From<ValueError> for PluginError {
    fn from(error: ValueError) -> Self {
        match error {
            ValueError::DoesNotExist(msg) => PluginError {
                body: msg,
                http_status_code: 404,
            },
            ValueError::Failure(msg) => PluginError {
                body: msg,
                http_status_code: 500,
            },
        }
    }
}

impl From<SetValueError> for PluginError {
    fn from(error: SetValueError) -> Self {
        match error {
            SetValueError::DoesNotExist(msg) => PluginError {
                body: msg,
                http_status_code: 404,
            },
            SetValueError::Failure(msg) => PluginError {
                body: msg,
                http_status_code: 500,
            },
        }
    }
}

/// An error returned by a failed Executor thread.
#[derive(Debug)]
pub struct ExecutorError {}

impl Error for ExecutorError {}

impl fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The executor thread failed")
    }
}

/// Represents the state of a result obtained by fetching a name from an attribute.
#[derive(Debug, PartialEq)]
pub enum NameError {
    DoesNotExist(String),
    Failure(String),
}

/// Represents the state of a result obtained by fetching a value from an attribute.
#[derive(Debug, PartialEq)]
pub enum ValueError {
    DoesNotExist(String),
    Failure(String),
}

/// Represents the state of a result obtained by setting a value of an attribute.
#[derive(Debug, PartialEq)]
pub enum SetValueError {
    DoesNotExist(String),
    Failure(String),
}
