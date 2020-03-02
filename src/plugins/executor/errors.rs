//! Error types for the executor module.

use std::{boxed::Box, error::Error, fmt, fmt::Debug};

use crate::models::ModelError;

/// An error returned when an operation in an executor fails.
#[derive(Debug)]
pub struct ExecutorError {
    /// The body of the HTTP response to return to the client.
    body: String,

    /// The HTTP status code to return to HTTP clients.
    http_status_code: u16,

    /// The cause of the error, if any.
    side: Option<Box<dyn Error + 'static + Send>>,
}

impl ExecutorError {
    pub fn new(
        body: String,
        http_status_code: u16,
        side: Option<Box<dyn Error + 'static + Send>>,
    ) -> ExecutorError {
        ExecutorError {
            body,
            http_status_code,
            side,
        }
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn http_status_code(&self) -> u16 {
        self.http_status_code
    }
}

impl Error for ExecutorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // The `as &_` is necessary for successful type inference due to the Send trait.
        // https://users.rust-lang.org/t/question-about-error-source-s-static-return-type/34515/7
        self.side.as_ref().map(|e| e.as_ref() as &_)
    }
}

impl fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ExecutorError {{ Cause: {:?} }}", self.side)
    }
}

impl PartialEq for ExecutorError {
    fn eq(&self, other: &Self) -> bool {
        match (self.side.as_ref(), other.side.as_ref()) {
            (None, None) => true,
            (Some(self_side), Some(other_side)) => {
                let self_side = format!("{}", self_side);
                let other_side = format!("{}", other_side);

                self_side == other_side
            }
            _ => false,
        }
    }
}

impl From<AdvancePhaseError> for ExecutorError {
    fn from(error: AdvancePhaseError) -> ExecutorError {
        ExecutorError::new(
            "Could not advance the phase of the of the plugin".to_string(),
            500,
            Some(Box::new(error)),
        )
    }
}

impl From<CountError> for ExecutorError {
    fn from(error: CountError) -> ExecutorError {
        ExecutorError::new(
            "Could not determine the number of plugin attributes".to_string(),
            500,
            Some(Box::new(error)),
        )
    }
}

impl From<IdsError> for ExecutorError {
    fn from(error: IdsError) -> ExecutorError {
        ExecutorError::new(
            "Could not determine the plugin's attribute IDs".to_string(),
            500,
            Some(Box::new(error)),
        )
    }
}

impl From<InitError> for ExecutorError {
    fn from(error: InitError) -> ExecutorError {
        ExecutorError::new(
            "Could not initialize the plugin".to_string(),
            500,
            Some(Box::new(error)),
        )
    }
}

impl From<NameError> for ExecutorError {
    fn from(error: NameError) -> ExecutorError {
        let (body, http_status_code) = match error {
            NameError::DoesNotExist(ref msg) => (msg.clone(), 404),
            NameError::Failure(ref msg) => (msg.clone(), 500),
        };
        ExecutorError::new(body, http_status_code, Some(Box::new(error)))
    }
}

impl From<PreInitError> for ExecutorError {
    fn from(error: PreInitError) -> ExecutorError {
        ExecutorError::new(
            "Could not determine pre-init status of the attribute".to_string(),
            500,
            Some(Box::new(error)),
        )
    }
}

impl From<SetValueError> for ExecutorError {
    fn from(error: SetValueError) -> ExecutorError {
        let (body, http_status_code) = match error {
            SetValueError::DoesNotExist(ref msg) => (msg.clone(), 404),
            SetValueError::Failure(ref msg) => (msg.clone(), 500),
            SetValueError::NotSettable(ref msg) => (msg.clone(), 422),
        };
        ExecutorError::new(body, http_status_code, Some(Box::new(error)))
    }
}

impl From<ModelError> for ExecutorError {
    fn from(error: ModelError) -> ExecutorError {
        ExecutorError::new(
            "Could not synchronize the plugin to the peripheral data".to_string(),
            500,
            Some(Box::new(error)),
        )
    }
}

impl From<ValueError> for ExecutorError {
    fn from(error: ValueError) -> ExecutorError {
        let (body, http_status_code) = match error {
            ValueError::DoesNotExist(ref msg) => (msg.clone(), 404),
            ValueError::Failure(ref msg) => (msg.clone(), 500),
        };
        ExecutorError::new(body, http_status_code, Some(Box::new(error)))
    }
}

impl From<std::str::Utf8Error> for ExecutorError {
    fn from(error: std::str::Utf8Error) -> Self {
        ExecutorError::new(
            "Could not convert the plugin's error message to a UTF8 string".to_string(),
            500,
            Some(Box::new(error)),
        )
    }
}

/// Represents an error which prevents the advance of the plugin's lifecycle phase.
#[derive(Debug)]
pub struct AdvancePhaseError(pub i32);

impl Error for AdvancePhaseError {}

impl fmt::Display for AdvancePhaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cannot advance from current phase: {}", self.0)
    }
}

/// Represents an error encountered when fetching the attribute count.
#[derive(Debug, PartialEq)]
pub struct CountError(pub String);

impl Error for CountError {}

impl fmt::Display for CountError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CountError {{ {} }}", self.0)
    }
}

/// Represents an error encountered when fetching the attribute IDs.
#[derive(Debug, PartialEq)]
pub struct IdsError(pub String);

impl Error for IdsError {}

impl fmt::Display for IdsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IdsError {}", self.0)
    }
}

/// An error raised during the plugin's initialization routine.
#[derive(Debug)]
pub struct InitError(pub String);

impl Error for InitError {}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InitError: {}", self.0)
    }
}

/// Represents the state of a result obtained by fetching a name from an attribute.
#[derive(Debug, PartialEq)]
pub enum NameError {
    DoesNotExist(String),
    Failure(String),
}

impl Error for NameError {}

impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NameError: {:?}", self)
    }
}

/// Represents the state of a result obtained by determining whether an attribute is pre-init.
#[derive(Debug, PartialEq)]
pub enum PreInitError {
    DoesNotExist(String),
    Failure(String),
}

impl Error for PreInitError {}

impl fmt::Display for PreInitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PreInitError: {:?}", self)
    }
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

/// Represents the state of a result obtained by fetching a value from an attribute.
#[derive(Debug, PartialEq)]
pub enum ValueError {
    DoesNotExist(String),
    Failure(String),
}

impl Error for ValueError {}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ValueError: {:?}", self)
    }
}
