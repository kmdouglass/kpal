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
    init::transmitters::Transmitters,
    models::Library,
    plugins::{
        messaging::{Message, Transmitter},
        PluginError,
    },
};

/// Result type containing a RequestHandlerError for the Err variant.
pub type Result<T> = std::result::Result<T, RequestHandlerError>;

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

/// An error raised when processing a request.
#[derive(Debug)]
pub struct RequestHandlerError {
    pub body: String,
    pub http_status_code: u16,
}

impl Error for RequestHandlerError {}

impl fmt::Display for RequestHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "RequestHandlerError {{ http_status_code: {}, body: {} }}",
            &self.http_status_code, &self.body
        )
    }
}

impl From<JsonError> for RequestHandlerError {
    fn from(error: JsonError) -> Self {
        RequestHandlerError {
            body: format!("Error when serializing to JSON: {}", error),
            http_status_code: 500,
        }
    }
}

impl From<ResourceNotFoundError> for RequestHandlerError {
    fn from(error: ResourceNotFoundError) -> Self {
        RequestHandlerError {
            body: format!("Error when accessing a resource: {}", error),
            http_status_code: 404,
        }
    }
}

impl From<PluginError> for RequestHandlerError {
    fn from(error: PluginError) -> Self {
        RequestHandlerError {
            body: error.body,
            http_status_code: error.http_status_code,
        }
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, Library>>> for RequestHandlerError {
    fn from(error: PoisonError<MutexGuard<Library>>) -> Self {
        RequestHandlerError {
            body: format!("Library mutex is poisoned: {}", error),
            http_status_code: 500,
        }
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, Transmitter>>> for RequestHandlerError {
    fn from(error: PoisonError<MutexGuard<Transmitter>>) -> Self {
        RequestHandlerError {
            body: format!("Peripheral thread is poisoned: {}", error),
            http_status_code: 500,
        }
    }
}

impl<'a> From<PoisonError<RwLockReadGuard<'a, Transmitters>>> for RequestHandlerError {
    fn from(error: PoisonError<RwLockReadGuard<Transmitters>>) -> Self {
        RequestHandlerError {
            body: format!("Transmitters thread is poisoned: {}", error),
            http_status_code: 500,
        }
    }
}

impl From<RecvTimeoutError> for RequestHandlerError {
    fn from(error: RecvTimeoutError) -> Self {
        RequestHandlerError {
            body: format!("Timeout while waiting on peripheral: {}", error),
            http_status_code: 500,
        }
    }
}

impl From<SendError<Message>> for RequestHandlerError {
    fn from(error: SendError<Message>) -> Self {
        RequestHandlerError {
            body: format!("Unable to send message to peripheral: {}", error),
            http_status_code: 500,
        }
    }
}
