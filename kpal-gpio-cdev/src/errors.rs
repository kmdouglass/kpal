//! Error implementations for the gpio-cdev plugin.

use std::boxed::Box;
use std::fmt;

use gpio_cdev::errors::ErrorKind;
use libc::c_int;

use kpal_plugin::error_codes::*;
use kpal_plugin::{Error as KpalError, PluginError};

/// Information passed to the caller when the GPIO plugin raises an error.
#[derive(Debug)]
pub struct GPIOPluginError {
    pub error_code: c_int,
    pub side: Option<Box<dyn std::error::Error + 'static>>,
}

impl std::error::Error for GPIOPluginError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.side.as_ref().map(|e| e.as_ref() as &_)
    }
}

impl fmt::Display for GPIOPluginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cause = match &self.side {
            Some(e) => format!("\nCaused by: {}", e),
            None => String::from(""),
        };
        write!(
            f,
            "GPIOPluginError: error code: {}{}",
            self.error_code, cause
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
            error_code: CONVERSION_ERR,
            side: Some(Box::new(error)),
        }
    }
}

impl From<std::num::TryFromIntError> for GPIOPluginError {
    fn from(error: std::num::TryFromIntError) -> GPIOPluginError {
        GPIOPluginError {
            error_code: CONVERSION_ERR,
            side: Some(Box::new(error)),
        }
    }
}

impl From<KpalError> for GPIOPluginError {
    fn from(error: KpalError) -> GPIOPluginError {
        let error_code = match error {
            KpalError::PluginUninitialized => PLUGIN_UNINIT_ERR,
            KpalError::ValueConversionError(_) => CONVERSION_ERR,
        };

        GPIOPluginError {
            error_code,
            side: Some(Box::new(error)),
        }
    }
}
