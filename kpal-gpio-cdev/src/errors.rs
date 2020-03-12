//! Error implementations for the gpio-cdev plugin.

use std::boxed::Box;
use std::error::Error;
use std::fmt;

use gpio_cdev::errors::ErrorKind;
use libc::c_int;

use kpal_plugin::error_codes::*;
use kpal_plugin::{PluginError, PluginUninitializedError};

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
    fn new(error_code: c_int) -> GPIOPluginError {
        GPIOPluginError {
            error_code,
            side: None,
        }
    }

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
            error_code,
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

impl From<std::ffi::IntoStringError> for GPIOPluginError {
    fn from(error: std::ffi::IntoStringError) -> GPIOPluginError {
        GPIOPluginError {
            error_code: STRING_CONVERSION_ERR,
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

impl From<PluginUninitializedError> for GPIOPluginError {
    fn from(error: PluginUninitializedError) -> GPIOPluginError {
        GPIOPluginError {
            error_code: PLUGIN_INIT_ERR,
            side: Some(Box::new(error)),
        }
    }
}
