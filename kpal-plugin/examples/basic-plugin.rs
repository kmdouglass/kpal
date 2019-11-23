//! A basic example of a plugin library with data but no hardware routines.
//!
//! This example demonstrates how to write a minimal plugin library that is capable of
//! communicating with a fake peripheral consisting of only a Rust struct. It does not communicate
//! with any actual hardware. Its primary purpose is for demonstration and testing.
//!
//! A library such as this one consists of four parts:
//!
//! 1. a struct which contains the peripheral's data
//! 2. a set of methods that the struct implements for manipulating the data and communicating with
//! the peripheral
//! 3. initialization routines that are exposed through the C API
//! 4. a set of functions that comprise the plugin API
// Import any needed items from the standard and 3rd party libraries.
use std::boxed::Box;
use std::convert::TryInto;
use std::ffi::{CStr, CString};
use std::ptr::null;

use libc::{c_int, c_uchar, size_t};

// Import the tools provided by the plugin library.
use kpal_plugin::constants::*;
use kpal_plugin::strings::copy_string;
use kpal_plugin::Value::*;
use kpal_plugin::*;

/// The first component of a plugin library is a struct that contains the peripheral's data.
///
/// In this example, the struct contains only one field, `attributes`, which contains the list of
/// all peripheral attributes provided by the plugin. In general, it can contain any number of
/// fields that are necessary for the plugin.
#[derive(Debug)]
#[repr(C)]
struct Basic {
    /// A Vec of attributes that describe the peripheral's state.
    attributes: Vec<Attribute>,
}

impl Basic {
    /// Returns a new instance of the peripheral.
    fn new() -> Basic {
        Basic {
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
        }
    }

    // The following methods that are implementend by the struct would normally communicate with
    // the peripheral. In this example, they simply return the values stored within the struct.
    /// Returns the name of an attribute.
    ///
    /// If the attribute that corresponds to the `id` does not exist, then an `AttributeError`
    /// instance is embedded in the return type. Otherwise, the name is returned as a C-compatible
    /// `&CStr`.
    ///
    /// # Arguments
    ///
    /// * `id` - the numeric ID of the attribute
    fn attribute_name(&self, id: usize) -> Result<&CStr> {
        log::debug!("Received request for the name of attribute: {}", id);
        match self.attributes.get(id) {
            Some(attribute) => Ok(&attribute.name),
            None => Err(AttributeError::new(
                Action::Get,
                ATTRIBUTE_DOES_NOT_EXIST,
                &format!("Attribute at index {} does not exist.", id),
            )),
        }
    }

    /// Returns the value of an attribute.
    ///
    /// If the attribute that corresponds to the `id` does not exist, then an `AttributeError`
    /// instance is embedded in the return type. Otherwise, the value is returnd as a C-compatible
    /// tagged enum.
    ///
    /// # Arguments
    ///
    /// * `id` - the numeric ID of the attribute
    fn attribute_value(&self, id: usize) -> Result<Value> {
        log::debug!("Received request for the value of attribute: {}", id);
        let attribute = self.attributes.get(id).ok_or_else(|| {
            AttributeError::new(
                Action::Get,
                ATTRIBUTE_DOES_NOT_EXIST,
                &format!("Attribute at index {} does not exist.", id),
            )
        })?;

        Ok(attribute.value.clone())
    }

    /// Sets the value of the attribute given by the id.
    ///
    /// If the attribute that corresponds to the `id` does not exist, or if the attribute cannot be
    /// set, then an `AttributeError` instance is embedded in the return type.
    ///
    /// # Arguments
    ///
    /// * `id` - the numeric ID of the attribute
    /// * `value` - a reference to a value
    fn attribute_set_value(&mut self, id: usize, value: &Value) -> Result<()> {
        log::debug!("Received request to set the value of attribute: {}", id);
        let current_value = &mut self
            .attributes
            .get_mut(id)
            .ok_or_else(|| {
                AttributeError::new(
                    Action::Get,
                    ATTRIBUTE_DOES_NOT_EXIST,
                    &format!("Attribute at index {} does not exist.", id),
                )
            })?
            .value;

        match (&current_value, &value) {
            (Int(_), Int(_)) | (Float(_), Float(_)) => {
                *current_value = (*value).clone();
                Ok(())
            }
            _ => Err(AttributeError::new(
                Action::Set,
                ATTRIBUTE_TYPE_MISMATCH,
                &format!("Attribute types do not match {}", id),
            )),
        }
    }
}

// The following functions are required. They are used by the daemon to initialize the library and
// new plugin instances, as well as to provide error information back to the daemon.
/// Initializes the library.
///
/// This function is called only once by the daemon. It is called when a library is first loaded
/// into memory.
#[no_mangle]
pub extern "C" fn kpal_library_init() -> c_int {
    env_logger::init();
    PLUGIN_OK
}

/// Returns a new Plugin instance containing the peripheral data and the function vtable.
///
/// The plugin is used by the daemon to communicate with the peripheral. It contains an opaque
/// pointer to the peripheral and a vtable. The vtable is a struct of function pointers to the
/// remaining functions in the plugin API.
#[no_mangle]
pub extern "C" fn kpal_plugin_init() -> Plugin {
    let peripheral: Box<Basic> = Box::new(Basic::new());
    let peripheral = Box::into_raw(peripheral) as *mut Peripheral;

    let vtable = VTable {
        peripheral_free: peripheral_free,
        error_message: error_message,
        attribute_name: attribute_name,
        attribute_value: attribute_value,
        set_attribute_value: set_attribute_value,
    };

    let plugin = Plugin {
        peripheral: peripheral,
        vtable: vtable,
    };

    log::debug!("Initialized plugin: {:?}", plugin);
    plugin
}

// The following functions are required. They are function pointers that belong to a vtable that
// defines the public plugin API. The functions that are pointed to are directly called by the
// daemon and wrap the methods that are implemented by the peripheral struct.
/// Frees the memory associated with the peripheral.
///
/// This routine will be called automatically by the daemon and should not be called by any user
/// code.
///
/// # Arguments
///
/// * `peripheral` - A pointer to a peripheral struct
extern "C" fn peripheral_free(peripheral: *mut Peripheral) {
    if peripheral.is_null() {
        return;
    }
    let peripheral = peripheral as *mut Box<Peripheral>;
    unsafe {
        Box::from_raw(peripheral);
    }
}

/// Returns an error message to the daemon given an error code.
///
/// If an undefined error code is provided, then this function will return a null pointer.
pub extern "C" fn error_message(error_code: c_int) -> *const c_uchar {
    let error_code: size_t = match error_code.try_into() {
        Ok(error_code) => error_code,
        Err(_) => {
            log::error!("Unrecognized error code provided");
            return null();
        }
    };

    ERRORS.get(error_code).map_or(null(), |e| e.as_ptr())
}

/// Writes the name of an attribute to a buffer that is provided by the caller.
///
/// This function returns a status code that indicates whether the operation succeeded and the
/// cause of any possible errors.
///
/// # Arguments
///
/// * `peripheral` - A pointer to a peripheral struct
/// * `id` - The id of the attribute
/// * `buffer` - A buffer of bytes into which the attribute's name will be written
/// * `length` - The length of the buffer
extern "C" fn attribute_name(
    peripheral: *const Peripheral,
    id: size_t,
    buffer: *mut c_uchar,
    length: size_t,
) -> c_int {
    //TODO Get rid of asserts
    assert!(!peripheral.is_null());
    let peripheral = peripheral as *const Basic;
    unsafe {
        let name: &[u8] = match (*peripheral).attribute_name(id) {
            Ok(name) => name.to_bytes_with_nul(),
            Err(e) => return e.error_code(),
        };

        match copy_string(name, buffer, length) {
            Ok(_) => PLUGIN_OK,
            Err(_) => UNDEFINED_ERR,
        }
    }
}

/// Writes the value of an attribute to a Value instance that is provided by the caller.
///
/// This function returns a status code that indicates whether the operation succeeded and the
/// cause of any possible errors.
///
/// # Arguments
///
/// * `peripheral` - A pointer to a peripheral struct
/// * `id` - The id of the attribute
/// * `value` - A pointer to a Value enum. The enum is provided by this function's caller.
extern "C" fn attribute_value(
    peripheral: *const Peripheral,
    id: size_t,
    value: *mut Value,
) -> c_int {
    //TODO Get rid of asserts
    assert!(!peripheral.is_null());
    let peripheral = peripheral as *const Basic;

    unsafe {
        match (*peripheral).attribute_value(id) {
            Ok(new_value) => {
                log::debug!(
                    "Response for the value of attribute {}: {:?}",
                    id,
                    new_value
                );
                *value = new_value
            }
            Err(e) => return e.error_code(),
        };
    }

    PLUGIN_OK
}

/// Sets the value of an attribute.
///
/// This function returns a status code that indicates whether the operation succeeded and the
/// cause of any possible errors.
///
/// # Arguments
///
/// * `peripheral` - A pointer to a peripheral struct
/// * `id` - The id of the attribute
/// * `value` - A pointer to a Value enum. The enum is provided by this function's caller and will
/// be copied.
extern "C" fn set_attribute_value(
    peripheral: *mut Peripheral,
    id: size_t,
    value: *const Value,
) -> c_int {
    if peripheral.is_null() || value.is_null() {
        return UNDEFINED_ERR;
    }
    let peripheral = peripheral as *mut Basic;

    unsafe {
        match (*peripheral).attribute_set_value(id, &*value) {
            Ok(_) => PLUGIN_OK,
            Err(e) => e.error_code(),
        }
    }
}

#[cfg(test)]
mod tests {
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
        let mut peripheral = Basic::new();
        let new_values = vec![Value::Float(3.14), Value::Int(4)];

        // Test setting each attribute to the new value
        for (i, value) in new_values.into_iter().enumerate() {
            peripheral.attribute_set_value(i, &value).unwrap();
            let actual = &peripheral.attributes[i].value;
            assert_eq!(
                value, *actual,
                "Expected attribute value to be {:?} but it was {:?}",
                value, *actual
            )
        }
    }

    #[test]
    fn set_attribute_wrong_variant() {
        let mut peripheral = Basic::new();
        let new_value = Value::Float(42.0);

        let result = peripheral.attribute_set_value(1, &new_value);
        match result {
            Ok(_) => panic!("Expected different value variants."),
            Err(_) => (),
        }
    }
}
