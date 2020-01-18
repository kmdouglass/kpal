//! Error types for the plugins module.

use std::{
    error::Error,
    fmt,
    fmt::Debug,
    sync::{MutexGuard, PoisonError, RwLockWriteGuard},
};

use crate::models::{AttributeError, ValueConversionError};
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

impl From<AdvancePhaseError> for PluginError {
    fn from(_error: AdvancePhaseError) -> Self {
        PluginError {
            body: "Could not advance the plugin's lifecycle phase".to_string(),
            http_status_code: 500,
        }
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

impl From<InitError> for PluginError {
    fn from(_error: InitError) -> Self {
        PluginError {
            body: "Could not initialize peripheral".to_string(),
            http_status_code: 500,
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

impl From<std::str::Utf8Error> for PluginError {
    fn from(_: std::str::Utf8Error) -> Self {
        PluginError {
            body: "Could not convert the plugin's error message to a UTF8 string".to_string(),
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

impl From<SyncError> for PluginError {
    fn from(_: SyncError) -> Self {
        PluginError {
            body: "Could not synchronize the plugin to the peripheral data".to_string(),
            http_status_code: 500,
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
            SetValueError::NotSettable(msg) => PluginError {
                body: msg,
                http_status_code: 422,
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

/// Represents an error which prevents the advance of the plugin's lifecycle phase.
#[derive(Debug)]
pub struct AdvancePhaseError {
    pub phase: i32,
}

impl Error for AdvancePhaseError {}

impl fmt::Display for AdvancePhaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cannot advance from current phase: {}", self.phase)
    }
}

/// /// An error raised during the plugin's initialization routine.
#[derive(Debug)]
pub struct InitError {
    pub msg: String,
}

impl Error for InitError {}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InitError: {:?}", self)
    }
}

/// An error raised when a plugin could not by synchronized to a peripheral.
#[derive(Debug)]
pub struct SyncError {
    pub side: Box<dyn Error>,
}

impl Error for SyncError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.side)
    }
}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SyncError: {:?}", self)
    }
}

impl From<ValueConversionError> for SyncError {
    fn from(error: ValueConversionError) -> Self {
        SyncError {
            side: Box::new(error),
        }
    }
}

/// Represents the state of a result obtained by fetching a name from an attribute.
#[derive(Debug, PartialEq)]
pub enum NameError {
    DoesNotExist(String),
    Failure(String),
}

/// Represents the state of a result obtained by determining whether an attribute is pre-init.
#[derive(Debug, PartialEq)]
pub enum PreInitError {
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
    NotSettable(String),
}

impl Error for SetValueError {}

impl fmt::Display for SetValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SetValueError: {:?}", self)
    }
}
