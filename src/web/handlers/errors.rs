//! Error types for the handlers module.
use std::{
    error::Error,
    fmt,
    sync::{
        mpsc::{RecvTimeoutError, SendError},
        MutexGuard, PoisonError, RwLockReadGuard,
    },
};

use rouille::input::json::JsonError;

use crate::{
    init::Transmitters,
    models::Library,
    plugins::{Message, PluginError, Transmitter},
};

/// An error raised when processing a request.
#[derive(Debug)]
pub struct HandlerError {
    pub body: String,
    pub http_status_code: u16,
}

impl Error for HandlerError {}

impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "HandlerError {{ http_status_code: {}, body: {} }}",
            &self.http_status_code, &self.body
        )
    }
}

impl From<JsonError> for HandlerError {
    fn from(error: JsonError) -> Self {
        HandlerError {
            body: format!("Error when serializing to JSON: {}", error),
            http_status_code: 500,
        }
    }
}

impl From<ResourceNotFoundError> for HandlerError {
    fn from(error: ResourceNotFoundError) -> Self {
        HandlerError {
            body: format!("Error when accessing a resource: {}", error),
            http_status_code: 404,
        }
    }
}

impl From<PluginError> for HandlerError {
    fn from(error: PluginError) -> Self {
        HandlerError {
            body: error.body,
            http_status_code: error.http_status_code,
        }
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, Library>>> for HandlerError {
    fn from(error: PoisonError<MutexGuard<Library>>) -> Self {
        HandlerError {
            body: format!("Library mutex is poisoned: {}", error),
            http_status_code: 500,
        }
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, Transmitter>>> for HandlerError {
    fn from(error: PoisonError<MutexGuard<Transmitter>>) -> Self {
        HandlerError {
            body: format!("Peripheral thread is poisoned: {}", error),
            http_status_code: 500,
        }
    }
}

impl<'a> From<PoisonError<RwLockReadGuard<'a, Transmitters>>> for HandlerError {
    fn from(error: PoisonError<RwLockReadGuard<Transmitters>>) -> Self {
        HandlerError {
            body: format!("Transmitters thread is poisoned: {}", error),
            http_status_code: 500,
        }
    }
}

impl From<RecvTimeoutError> for HandlerError {
    fn from(error: RecvTimeoutError) -> Self {
        HandlerError {
            body: format!("Timeout while waiting on peripheral: {}", error),
            http_status_code: 500,
        }
    }
}

impl From<SendError<Message>> for HandlerError {
    fn from(error: SendError<Message>) -> Self {
        HandlerError {
            body: format!("Unable to send message to peripheral: {}", error),
            http_status_code: 500,
        }
    }
}

/// An error raised when a peripheral is not found.
#[derive(Debug)]
pub struct ResourceNotFoundError {
    /// The id of the resource
    pub id: usize,

    /// The name of the resource collection (e.g. peripherals)
    pub name: String,
}

impl Error for ResourceNotFoundError {}

impl fmt::Display for ResourceNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Resource not found: {}/{}", self.name, self.id)
    }
}
