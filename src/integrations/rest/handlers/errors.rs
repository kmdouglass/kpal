//! Error types for the handlers module.
use std::{
    boxed::Box,
    error::Error,
    fmt,
    sync::{MutexGuard, PoisonError},
};

use {rouille::input::json::JsonError, serde::Serialize};

use crate::{
    integrations::rest::status_from_reason, integrations::IntegrationsError, models::Library,
};

use super::super::schemas::SchemaError;

/// An error raised when processing a request.
#[derive(Debug, Serialize)]
pub struct RestHandlerError {
    pub message: String,

    #[serde(skip)]
    pub http_status_code: u16,

    #[serde(skip)]
    pub side: Option<Box<dyn Error + 'static>>,
}

impl Error for RestHandlerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.side.as_ref().map(|e| e.as_ref())
    }
}

impl fmt::Display for RestHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "RestHandlerError {{ http_status_code: {}, message: {}, side: {:?} }}",
            &self.http_status_code,
            &self.message,
            self.source()
        )
    }
}

impl From<JsonError> for RestHandlerError {
    fn from(error: JsonError) -> RestHandlerError {
        RestHandlerError {
            message: format!("Error when deserializing JSON: {}", error),
            http_status_code: 400,
            side: Some(Box::new(error)),
        }
    }
}

impl From<IntegrationsError> for RestHandlerError {
    fn from(error: IntegrationsError) -> RestHandlerError {
        RestHandlerError {
            message: format!("Error from the KPAL core API: {}", error.message()),
            http_status_code: status_from_reason(error.reason()),
            side: Some(Box::new(error)),
        }
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, Library>>> for RestHandlerError {
    fn from(error: PoisonError<MutexGuard<Library>>) -> RestHandlerError {
        RestHandlerError {
            message: format!("Library mutex is poisoned: {}", error),
            http_status_code: 500,
            side: None,
        }
    }
}

impl From<SchemaError> for RestHandlerError {
    fn from(error: SchemaError) -> RestHandlerError {
        RestHandlerError {
            message: format!("Error processing data in the REST integration: {}", error),
            http_status_code: 422,
            side: Some(Box::new(error)),
        }
    }
}
