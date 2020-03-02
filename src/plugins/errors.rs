//! Error types for the plugins module.

use std::{
    error::Error,
    fmt,
    fmt::Debug,
    sync::{MutexGuard, PoisonError, RwLockWriteGuard},
};

use crate::models::ModelError;
use crate::{init::Transmitters, models::Library};

use super::executor::ExecutorError;

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

impl From<ModelError> for PluginError {
    fn from(_error: ModelError) -> Self {
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

impl From<ExecutorError> for PluginError {
    fn from(error: ExecutorError) -> Self {
        PluginError {
            body: format!("{}", error),
            http_status_code: error.http_status_code(),
        }
    }
}

impl From<MergeAttributesError> for PluginError {
    fn from(error: MergeAttributesError) -> Self {
        match error {
            MergeAttributesError::DoesNotExist(msg) => PluginError {
                body: msg,
                http_status_code: 404,
            },
            MergeAttributesError::Failure(msg) => PluginError {
                body: msg,
                http_status_code: 500,
            },
            MergeAttributesError::IsNotPreInit(msg) => PluginError {
                body: msg,
                http_status_code: 422,
            },
            MergeAttributesError::UnknownVariant(msg) => PluginError {
                body: msg,
                http_status_code: 500,
            },
            MergeAttributesError::VariantMismatch(msg) => PluginError {
                body: msg,
                http_status_code: 422,
            },
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

/// Raised when the user-provided attribute values cannot be merged into the defaults.
#[derive(Debug)]
pub enum MergeAttributesError {
    DoesNotExist(String),
    Failure(String),
    IsNotPreInit(String),
    UnknownVariant(String),
    VariantMismatch(String),
}

impl Error for MergeAttributesError {}

impl fmt::Display for MergeAttributesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MergeAttributesError: {:?}", self)
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, Library>>> for MergeAttributesError {
    fn from(_error: PoisonError<MutexGuard<Library>>) -> Self {
        MergeAttributesError::Failure("The thread's mutex is poisoned.".to_string())
    }
}
