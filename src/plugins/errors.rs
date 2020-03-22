//! Error types for the plugins module.

use std::{
    error::Error,
    fmt,
    fmt::Debug,
    sync::{MutexGuard, PoisonError, RwLockWriteGuard},
};

use crate::{
    init::Transmitters,
    integrations::ErrorReason,
    models::{Library, ModelError},
};

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
    /// The message of the HTTP response to return to the client.
    message: String,

    /// The reason for the error. This is used by integrations to translate into their own error
    /// responses.
    reason: ErrorReason,

    /// The lower-level instance of the Error that that caused this one, if any.
    side: Option<Box<dyn Error + 'static + Send>>,
}

impl PluginError {
    pub fn new(
        message: String,
        reason: ErrorReason,
        side: Option<Box<dyn Error + 'static + Send>>,
    ) -> PluginError {
        PluginError {
            message,
            reason,
            side,
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn reason(&self) -> ErrorReason {
        self.reason
    }
}

impl Error for PluginError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // The `as &_` is necessary for successful type inference due to the Send trait.
        // https://users.rust-lang.org/t/question-about-error-source-s-static-return-type/34515/7
        self.side.as_ref().map(|e| e.as_ref() as &_)
    }
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PluginError: {:?}", self)
    }
}

impl From<ModelError> for PluginError {
    fn from(error: ModelError) -> Self {
        PluginError {
            message: "Could not create attribute from value".to_string(),
            reason: ErrorReason::InternalError,
            side: Some(Box::new(error)),
        }
    }
}

impl From<std::io::Error> for PluginError {
    fn from(error: std::io::Error) -> Self {
        PluginError {
            message: "Could not get symbol from shared library".to_string(),
            reason: ErrorReason::InternalError,
            side: Some(Box::new(error)),
        }
    }
}

impl From<ExecutorError> for PluginError {
    fn from(error: ExecutorError) -> Self {
        PluginError {
            message: format!("{}", error),
            reason: error.reason(),
            side: Some(Box::new(error)),
        }
    }
}

impl From<MergeAttributesError> for PluginError {
    fn from(error: MergeAttributesError) -> Self {
        let err2 = error.clone();
        match err2 {
            MergeAttributesError::Failure(msg) => PluginError {
                message: msg,
                reason: ErrorReason::InternalError,
                side: Some(Box::new(error)),
            },
            MergeAttributesError::IsNotPreInit(msg) => PluginError {
                message: msg,
                reason: ErrorReason::UnprocessableRequest,
                side: Some(Box::new(error)),
            },
            MergeAttributesError::VariantMismatch(msg) => PluginError {
                message: msg,
                reason: ErrorReason::UnprocessableRequest,
                side: Some(Box::new(error)),
            },
        }
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, Library>>> for PluginError {
    fn from(_error: PoisonError<MutexGuard<Library>>) -> Self {
        PluginError {
            message: "The Mutex on the library is poisoned".to_string(),
            reason: ErrorReason::InternalError,
            side: None,
        }
    }
}

impl<'a> From<PoisonError<RwLockWriteGuard<'a, Transmitters>>> for PluginError {
    fn from(_error: PoisonError<RwLockWriteGuard<Transmitters>>) -> Self {
        PluginError {
            message: "The RwLock on the transmitters collection is poisoned".to_string(),
            reason: ErrorReason::InternalError,
            side: None,
        }
    }
}

/// Raised when the user-provided attribute values cannot be merged into the defaults.
#[derive(Debug, Clone)]
pub enum MergeAttributesError {
    Failure(String),
    IsNotPreInit(String),
    VariantMismatch(String),
}

impl Error for MergeAttributesError {}

impl fmt::Display for MergeAttributesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MergeAttributesError: {:?}", self)
    }
}

impl From<ModelError> for MergeAttributesError {
    fn from(_error: ModelError) -> MergeAttributesError {
        MergeAttributesError::Failure("Could not build attribute from builder".to_string())
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, Library>>> for MergeAttributesError {
    fn from(_error: PoisonError<MutexGuard<Library>>) -> MergeAttributesError {
        MergeAttributesError::Failure("The thread's mutex is poisoned".to_string())
    }
}
