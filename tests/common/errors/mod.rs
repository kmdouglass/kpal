use std::{boxed::Box, error::Error, fmt, io};

use reqwest::Error as ReqwestError;

/// An error returned by any failure in the common test code.
#[derive(Debug)]
pub struct CommonError {
    side: Option<Box<dyn Error + 'static>>,
}

impl CommonError {
    pub fn new(side: Option<Box<dyn Error + 'static>>) -> CommonError {
        CommonError { side }
    }
}

impl Error for CommonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.side.as_ref().map(|e| e.as_ref())
    }
}

impl fmt::Display for CommonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CommonError {{ Cause: {:?} }}", self.side)
    }
}

impl From<io::Error> for CommonError {
    fn from(error: io::Error) -> CommonError {
        CommonError::new(Some(Box::new(error)))
    }
}

impl From<ReqwestError> for CommonError {
    fn from(error: ReqwestError) -> CommonError {
        CommonError::new(Some(Box::new(error)))
    }
}

/// Indicates that an error occured when starting the daemon.
#[derive(Debug)]
pub struct StartDaemonError {}

impl Error for StartDaemonError {}

impl fmt::Display for StartDaemonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StartDaemonError")
    }
}
