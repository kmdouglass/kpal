//! Error implementations for the gpio-cdev plugin.

use std::boxed::Box;
use std::error::Error;
use std::fmt;

use gpio_cdev::errors::ErrorKind;
use libc::c_int;

use kpal_plugin::constants::*;
use kpal_plugin::PluginError;

/// An error raised when trying to create a new GPIO plugin instance.
#[derive(Debug)]
pub struct GPIOPluginError {
    pub error_code: c_int,
    pub side: Option<Box<dyn Error + 'static>>,
}

impl Error for GPIOPluginError {}

impl fmt::Display for GPIOPluginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "GPIOPluginError {{ error_code: {}, side: {:?} }}",
            self.error_code, self.side
        )
    }
}

impl PluginError for GPIOPluginError {
    fn error_code(&self) -> c_int {
        self.error_code
    }
}

impl From<gpio_cdev::errors::Error> for GPIOPluginError {
    fn from(error: gpio_cdev::errors::Error) -> GPIOPluginError {
        let error_code = match error.kind() {
            ErrorKind::Io(_) => IO_ERR,
            _ => UNDEFINED_ERR,
        };

        GPIOPluginError {
            error_code: error_code,
            side: Some(Box::new(error)),
        }
    }
}

impl From<std::cell::BorrowMutError> for GPIOPluginError {
    fn from(error: std::cell::BorrowMutError) -> GPIOPluginError {
        GPIOPluginError {
            error_code: UNDEFINED_ERR,
            side: Some(Box::new(error)),
        }
    }
}

impl From<std::convert::Infallible> for GPIOPluginError {
    fn from(error: std::convert::Infallible) -> GPIOPluginError {
        GPIOPluginError {
            error_code: UNDEFINED_ERR,
            side: Some(Box::new(error)),
        }
    }
}

impl From<std::num::TryFromIntError> for GPIOPluginError {
    fn from(error: std::num::TryFromIntError) -> GPIOPluginError {
        GPIOPluginError {
            error_code: NUMERIC_CONVERSION_ERR,
            side: Some(Box::new(error)),
        }
    }
}
