use std::{
    boxed::Box,
    error::Error,
    fmt,
    sync::{
        mpsc::{RecvTimeoutError, SendError},
        MutexGuard, PoisonError, RwLockReadGuard,
    },
};

use crate::{
    init::Transmitters,
    models::Library,
    plugins::{Message, PluginError, Transmitter},
};

/// A reason for why an error occurred in a KPAL module.
///
/// This is used by integrations to determine their own error responses.
#[derive(Clone, Copy, Debug)]
pub enum ErrorReason {
    InternalError,
    ResourceNotFound,
    UnprocessableRequest,
}

/// An error that is raised when a top-level component of KPAL fails.
#[derive(Debug)]
pub struct IntegrationsError {
    /// A message to return to the user about why the error occurred.
    message: String,

    /// A reason for why the error occured.
    reason: ErrorReason,

    /// The lower-level error that was raised, if any.
    side: Option<Box<dyn Error + 'static>>,
}

impl IntegrationsError {
    /// Creates a new instance of an IntegrationsError.
    pub fn new(
        message: String,
        reason: ErrorReason,
        side: Option<Box<dyn Error + 'static>>,
    ) -> IntegrationsError {
        IntegrationsError {
            message,
            reason,
            side,
        }
    }

    /// Returns the error message associated with the error.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the reason for the error.
    pub fn reason(&self) -> ErrorReason {
        self.reason
    }
}

impl Error for IntegrationsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.side.as_ref().map(|e| e.as_ref())
    }
}

impl fmt::Display for IntegrationsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IntegrationsError {{ Cause: {:?} }}", self.side)
    }
}

impl From<PluginError> for IntegrationsError {
    fn from(error: PluginError) -> IntegrationsError {
        IntegrationsError::new(
            error.message().to_owned(),
            error.reason(),
            Some(Box::new(error)),
        )
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, Library>>> for IntegrationsError {
    fn from(_: PoisonError<MutexGuard<Library>>) -> IntegrationsError {
        let message = "Unable to retrieve the library because its thread is poisoned".to_string();
        IntegrationsError {
            message,
            reason: ErrorReason::InternalError,
            side: None, // The PoisonError contains an item with a non-static lifetime.
        }
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, Transmitter>>> for IntegrationsError {
    fn from(_: PoisonError<MutexGuard<Transmitter>>) -> Self {
        let message =
            "Unable to communicate with the peripheral transmitter because its thread is poisoned"
                .to_string();
        IntegrationsError {
            message,
            reason: ErrorReason::InternalError,
            side: None, // The PoisonError contains an item with a non-static lifetime.
        }
    }
}

impl<'a> From<PoisonError<RwLockReadGuard<'a, Transmitters>>> for IntegrationsError {
    fn from(_: PoisonError<RwLockReadGuard<Transmitters>>) -> IntegrationsError {
        let message =
            "Unable to communicate with the plugin because its thread is poisoned".to_string();
        IntegrationsError {
            message,
            reason: ErrorReason::InternalError,
            side: None, // The PoisonError contains an item with a non-static lifetime.
        }
    }
}

impl From<RecvTimeoutError> for IntegrationsError {
    fn from(error: RecvTimeoutError) -> Self {
        IntegrationsError {
            message: format!("Timeout while waiting on peripheral: {}", error),
            reason: ErrorReason::InternalError,
            side: Some(Box::new(error)),
        }
    }
}

impl From<SendError<Message>> for IntegrationsError {
    fn from(error: SendError<Message>) -> Self {
        IntegrationsError {
            message: format!("Unable to send message to peripheral: {}", error),
            reason: ErrorReason::InternalError,
            side: Some(Box::new(error)),
        }
    }
}
