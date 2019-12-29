//! A basic example of a plugin library with data but no hardware routines.
//!
//! This example demonstrates how to write a minimal plugin library that is capable of
//! communicating with a fake plugin consisting of only a Rust struct. It does not communicate with
//! any actual hardware. Its primary purpose is for demonstration and testing.
//!
//! A library such as this one consists of four parts:
//!
//! 1. a struct which contains the plugin's data
//! 2. a set of methods that the struct implements for manipulating the data and communicating with
//! the plugin
//! 3. initialization routines that are exposed through the C API
//! 4. a set of functions that comprise the plugin API
// Import any needed items from the standard and 3rd party libraries.
use std::{
    boxed::Box,
    error::Error,
    ffi::{CStr, CString},
    fmt,
};

use libc::c_int;

// Import the tools provided by the plugin library.
use kpal_plugin::{constants::*, Value::*, *};

/// The first component of a plugin library is a struct that contains the plugin's data.
///
/// In this example, the struct contains only one field, `attributes`, which contains the list of
/// all attributes provided by the plugin. In general, it can contain any number of fields that are
/// necessary for the plugin.
#[derive(Debug)]
#[repr(C)]
struct Basic {
    /// A Vec of attributes that describe the peripheral's state.
    attributes: Vec<Attribute>,
}

// Plugins implement the PluginAPI trait. They take a custom error type that is also provided by
// the library (see below) and an associated type that indicates the type that holds the plugin's
// data.
impl PluginAPI<BasicError> for Basic {
    type Plugin = Basic;

    /// Returns a new instance of the peripheral.
    fn new() -> Result<Basic, BasicError> {
        Ok(Basic {
            attributes: vec![
                Attribute {
                    name: CString::new("x").expect("Error creating CString"),
                    value: Value::Float(0.0),
                },
                Attribute {
                    name: CString::new("y").expect("Error creating CString"),
                    value: Value::Int(0),
                },
            ],
        })
    }

    // The following methods that are implementend by the struct would normally communicate with
    // the hardware device. In this example, they simply return the values stored within the
    // struct.
    /// Returns the name of an attribute.
    ///
    /// If the attribute that corresponds to the `id` does not exist, then an error is
    /// returned. Otherwise, the name is returned as a C-compatible `&CStr`.
    ///
    /// # Arguments
    ///
    /// * `id` - the numeric ID of the attribute
    fn attribute_name(&self, id: usize) -> Result<&CStr, BasicError> {
        log::debug!("Received request for the name of attribute: {}", id);
        match self.attributes.get(id) {
            Some(attribute) => Ok(&attribute.name),
            None => Err(BasicError {
                error_code: ATTRIBUTE_DOES_NOT_EXIST,
            }),
        }
    }

    /// Returns the value of an attribute.
    ///
    /// If the attribute that corresponds to the `id` does not exist, then an error is
    /// returned. Otherwise, the value is returnd as a C-compatible tagged enum.
    ///
    /// # Arguments
    ///
    /// * `id` - the numeric ID of the attribute
    fn attribute_value(&self, id: usize) -> Result<Value, BasicError> {
        log::debug!("Received request for the value of attribute: {}", id);
        let attribute = self.attributes.get(id).ok_or_else(|| BasicError {
            error_code: ATTRIBUTE_DOES_NOT_EXIST,
        })?;

        Ok(attribute.value.clone())
    }

    /// Sets the value of the attribute given by the id.
    ///
    /// If the attribute that corresponds to the `id` does not exist, or if the attribute cannot be
    /// set, then an error is returned.
    ///
    /// # Arguments
    ///
    /// * `id` - the numeric ID of the attribute
    /// * `value` - a reference to a value
    fn attribute_set_value(&mut self, id: usize, value: &Value) -> Result<(), BasicError> {
        log::debug!("Received request to set the value of attribute: {}", id);
        let current_value = &mut self
            .attributes
            .get_mut(id)
            .ok_or_else(|| BasicError {
                error_code: ATTRIBUTE_DOES_NOT_EXIST,
            })?
            .value;

        match (&current_value, &value) {
            (Int(_), Int(_)) | (Float(_), Float(_)) => {
                *current_value = (*value).clone();
                Ok(())
            }
            _ => Err(BasicError {
                error_code: ATTRIBUTE_TYPE_MISMATCH,
            }),
        }
    }
}

/// The plugin's error type.
///
/// Plugin methods all return the same, custom error type provided by the plugin author(s). This
/// allows developers to pack any information that they wish into the error type. In addition, by
/// providing their own error, plugin authors can implement the From trait to automatically
/// transform errors raised in the PluginAPI methods into this type with the `?` operator.
#[derive(Debug)]
struct BasicError {
    error_code: c_int,
}

impl Error for BasicError {}

impl fmt::Display for BasicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Basic error {{ error_code: {} }}", self.error_code)
    }
}

/// KPAL requires that the library's custom error implement the PluginError trait
///
/// This ensures that required information is always passed back to the daemon.
impl PluginError for BasicError {
    /// Returns the error code associated with the plugin's error.
    ///
    /// This code should be one of the values found in the `constants` module.
    fn error_code(&self) -> c_int {
        self.error_code
    }
}

// Now that everything has been setup, we call the `declare_plugin` macro to automatically generate
// the functions and structures that will be used by the daemon to communicate with the
// plugin. This macro allows plugin developers to entirely avoid writing unsafe, foreign function
// interface code.
declare_plugin!(Basic, BasicError);

// Unit tests for the plugin lie within a module called `tests` that is preceded by a #[cfg(test)]
// attribute.
#[cfg(test)]
mod tests {
    use libc::c_uchar;

    use super::*;

    #[test]
    fn test_kpal_error() {
        struct Case {
            description: &'static str,
            error_code: c_int,
            want_null: bool,
        };

        let cases = vec![
            Case {
                description: "a valid error code is passed to kpal_error",
                error_code: 0,
                want_null: false,
            },
            Case {
                description: "an invalid and negative error code is passed to kpal_error",
                error_code: -1,
                want_null: true,
            },
            Case {
                description: "an invalid and positive error code is passed to kpal_error",
                error_code: 99999,
                want_null: true,
            },
        ];

        let mut msg: *const c_uchar;
        for case in &cases {
            log::info!("{}", case.description);
            msg = error_message(case.error_code);

            if case.want_null {
                assert!(msg.is_null());
            } else {
                assert!(!msg.is_null());
            }
        }
    }

    #[test]
    fn set_attribute_value() {
        let mut plugin = Basic::new().unwrap();
        let new_values = vec![Value::Float(3.14), Value::Int(4)];

        // Test setting each attribute to the new value
        for (i, value) in new_values.into_iter().enumerate() {
            plugin.attribute_set_value(i, &value).unwrap();
            let actual = &plugin.attributes[i].value;
            assert_eq!(
                value, *actual,
                "Expected attribute value to be {:?} but it was {:?}",
                value, *actual
            )
        }
    }

    #[test]
    fn set_attribute_wrong_variant() {
        let mut plugin = Basic::new().unwrap();
        let new_value = Value::Float(42.0);

        let result = plugin.attribute_set_value(1, &new_value);
        match result {
            Ok(_) => panic!("Expected different value variants."),
            Err(_) => (),
        }
    }
}
