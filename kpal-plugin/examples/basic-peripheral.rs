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
use std::ffi::{CStr, CString};

use libc::{c_int, c_uchar, size_t};

// Import the tools provided by the plugin library.
use kpal_plugin::constants::*;
use kpal_plugin::strings::copy_string;
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
        match self.attributes.get(id) {
            Some(attribute) => Ok(&attribute.name),
            None => Err(AttributeError::new(
                Action::Get,
                PERIPHERAL_ATTRIBUTE_DOES_NOT_EXIST,
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
        let attribute = self.attributes.get(id).ok_or_else(|| {
            AttributeError::new(
                Action::Get,
                PERIPHERAL_ATTRIBUTE_DOES_NOT_EXIST,
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
        use Value::*;

        let current_value = &mut self
            .attributes
            .get_mut(id)
            .ok_or_else(|| {
                AttributeError::new(
                    Action::Get,
                    PERIPHERAL_ATTRIBUTE_DOES_NOT_EXIST,
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
                PERIPHERAL_COULD_NOT_SET_ATTRIBUTE,
                &format!("Could not set attribute {}", id),
            )),
        }
    }
}

// The following functions are required. They are used by the daemon to initialize the library and
// new plugin instances.
/// Initializes the library.
///
/// This function is called only once by the daemon. It is called when a library is first loaded
/// into memory.
#[no_mangle]
pub extern "C" fn kpal_library_init() -> c_int {
    env_logger::init();
    LIBRARY_OK
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
            Ok(_) => PERIPHERAL_OK,
            Err(_) => PERIPHERAL_ERR,
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
        log::debug!(
            "Received request for the value of attribute {} for peripheral: {:?}",
            id,
            *peripheral
        );
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

    PERIPHERAL_OK
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
        return PERIPHERAL_ERR;
    }
    let peripheral = peripheral as *mut Basic;

    unsafe {
        match (*peripheral).attribute_set_value(id, &*value) {
            Ok(_) => PERIPHERAL_OK,
            Err(e) => e.error_code(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
