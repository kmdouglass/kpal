use std::{boxed::Box, error::Error, fmt};

use super::schemas::SchemaError;

use crate::integrations::ErrorReason;

/// An error that is raised when a top-level component of the REST integration fails.
#[derive(Debug)]
pub struct RestIntegrationError {
    /// The cause of the error, if any.
    side: Option<Box<dyn Error + 'static>>,
}

impl Error for RestIntegrationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.side.as_ref().map(|e| e.as_ref())
    }
}

impl fmt::Display for RestIntegrationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RestIntegrationError {{ Cause: {:?} }}", self.side)
    }
}

impl From<SchemaError> for RestIntegrationError {
    fn from(error: SchemaError) -> RestIntegrationError {
        RestIntegrationError {
            side: Some(Box::new(error)),
        }
    }
}

/// Maps a reason for an error returned by the KPAL core onto an HTTP status code.
pub fn status_from_reason(reason: ErrorReason) -> u16 {
    use ErrorReason::*;

    match reason {
        InternalError => 500,
        ResourceNotFound => 404,
        UnprocessableRequest => 422,
    }
}
