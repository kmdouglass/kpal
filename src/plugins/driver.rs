//! Methods for communicating directly with Plugins.
use std::ffi::CStr;
use std::fmt;

use libc::{c_char, c_int, c_uchar, size_t};
use log;
use memchr::memchr;

use kpal_plugin::constants::*;
use kpal_plugin::Value;

use super::Plugin;

use crate::constants::*;

/// Returns the name of an attribute from a Plugin.
///
/// # Arguments
///
/// * `plugin` - A reference to the Plugin from which an attribute's name will be obtained
/// * `id` - The attribute's unique ID
pub fn attribute_name(plugin: &Plugin, id: size_t) -> Result<String, NameError> {
    let mut name = [0u8; ATTRIBUTE_NAME_BUFFER_LENGTH];

    let result = (plugin.vtable.attribute_name)(
        plugin.peripheral,
        id,
        &mut name[0] as *mut c_uchar,
        ATTRIBUTE_NAME_BUFFER_LENGTH,
    );

    if result == PLUGIN_OK {
        let name = match memchr(0, &name)
            .ok_or("could not find null byte")
            .and_then(|null_byte| {
                CStr::from_bytes_with_nul(&name[..=null_byte])
                    .map_err(|_| "could not convert name from C string")
            })
            .map(|name| name.to_string_lossy().into_owned())
        {
            Ok(name) => name,
            Err(err) => {
                log::error!("{}", err);
                String::from("Unknown")
            }
        };

        log::debug!("Received name: {:?}", name);
        Ok(name)
    } else if result == ATTRIBUTE_DOES_NOT_EXIST {
        log::debug!("Attribute does not exist: {}", result);
        let msg = unsafe { error_message(&plugin, result).unwrap_or(String::from("")) };
        Err(NameError::DoesNotExist(msg))
    } else {
        log::error!(
            "Received error code while getting attribute name: {}",
            result
        );
        let msg = unsafe { error_message(&plugin, result).unwrap_or(String::from("")) };
        Err(NameError::Failure(msg))
    }
}

/// Returns the value of an attribute from a Plugin.
///
/// # Arguments
///
/// * `plugin` - A reference to the Plugin from which an attribute will be obtained
/// * `id` - The attribute's unique ID
/// * `value` - A reference to a value instance into which the attribute's value will be copied
pub fn attribute_value(plugin: &Plugin, id: size_t, value: &mut Value) -> Result<(), ValueError> {
    let result = (plugin.vtable.attribute_value)(plugin.peripheral, id, value as *mut Value);

    if result == PLUGIN_OK {
        log::debug!("Received value: {:?}", value);
        Ok(())
    } else if result == ATTRIBUTE_DOES_NOT_EXIST {
        log::debug!("Attribute does not exist: {}", result);
        let msg = unsafe { error_message(&plugin, result).unwrap_or(String::from("")) };
        Err(ValueError::DoesNotExist(msg))
    } else {
        log::error!(
            "Received error code while fetching attribute value: {}",
            result
        );
        let msg = unsafe { error_message(&plugin, result).unwrap_or(String::from("")) };
        Err(ValueError::Failure(msg))
    }
}

/// Requests an error message from a plugin given an error code.
///
/// # Safety
///
/// This function is unsafe because it calls a function that is provided by the shared library
/// through the FFI.
///
/// # Arguments
///
/// * `lib` - A copy of the Library that contains the implementation of the peripheral's Plugin API
unsafe fn error_message(plugin: &Plugin, error_code: c_int) -> Result<String, KpalErrorMsg> {
    let msg_p = (plugin.vtable.error_message)(error_code) as *const c_char;

    let msg = if msg_p.is_null() {
        return Err(KpalErrorMsg {});
    } else {
        CStr::from_ptr(msg_p).to_str()?.to_owned()
    };

    Ok(msg)
}

/// Sets the value of an attribute of a Plugin.
///
/// # Arguments
///
/// * `plugin` - A reference to the Plugin on which the attribute will be set
/// * `id` - The attribute's unique ID
/// * `value` - A reference to a value instance that will be copied into the plugin
pub fn set_attribute_value(
    plugin: &Plugin,
    id: size_t,
    value: &Value,
) -> Result<(), SetValueError> {
    let result = (plugin.vtable.set_attribute_value)(plugin.peripheral, id, value as *const Value);

    if result == PLUGIN_OK {
        log::debug!("Set value: {:?}", value);
        Ok(())
    } else if result == ATTRIBUTE_DOES_NOT_EXIST {
        log::debug!("Attribute does not exist: {}", result);
        let msg = unsafe { error_message(&plugin, result).unwrap_or(String::from("")) };
        Err(SetValueError::DoesNotExist(msg))
    } else {
        log::error!(
            "Received error code while setting attribute value: {}",
            result
        );
        let msg = unsafe { error_message(&plugin, result).unwrap_or(String::from("")) };
        Err(SetValueError::Failure(msg))
    }
}

/// Represents a failure to recover an error message from the peripheral.
#[derive(Debug)]
struct KpalErrorMsg {}

impl fmt::Display for KpalErrorMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error retrieving error message from the peripheral")
    }
}

impl From<std::str::Utf8Error> for KpalErrorMsg {
    fn from(_: std::str::Utf8Error) -> Self {
        KpalErrorMsg {}
    }
}

/// Represents the state of a result obtained by fetching a name from an attribute.
#[derive(Debug, PartialEq)]
pub enum NameError {
    DoesNotExist(String),
    Failure(String),
}

/// Represents the state of a result obtained by fetching a value from an attribute.
#[derive(Debug, PartialEq)]
pub enum ValueError {
    DoesNotExist(String),
    Failure(String),
}

/// Represents the state of a result obtained by setting a value of an attribute.
#[derive(Debug, PartialEq)]
pub enum SetValueError {
    DoesNotExist(String),
    Failure(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::boxed::Box;

    use kpal_plugin::{Peripheral, Plugin, VTable, Value};
    use libc::{c_int, c_uchar, size_t};

    use crate::plugins::driver::{NameError, ValueError};

    #[test]
    fn test_error_message() {
        let plugin = set_up();
        let msg = unsafe { error_message(&plugin, 0) };
        assert_eq!("foo", msg.unwrap());
    }

    #[test]
    fn test_attribute_name() {
        let mut plugin = set_up();
        let cases: Vec<(
            Result<String, NameError>,
            extern "C" fn(*const Peripheral, size_t, *mut c_uchar, size_t) -> c_int,
        )> = vec![
            (Ok(String::from("")), attribute_name_ok),
            (
                Err(NameError::DoesNotExist(String::from("foo"))),
                attribute_name_does_not_exist,
            ),
            (
                Err(NameError::Failure(String::from("foo"))),
                attribute_name_failure,
            ),
        ];

        let mut result: Result<String, NameError>;
        for (expected, case) in cases {
            plugin.vtable.attribute_name = case;
            result = attribute_name(&plugin, 0);
            assert_eq!(expected, result);
        }

        tear_down(plugin);
    }

    #[test]
    fn test_attribute_value() {
        let mut plugin = set_up();
        let cases: Vec<(
            Result<(), ValueError>,
            extern "C" fn(*const Peripheral, size_t, *mut Value) -> c_int,
        )> = vec![
            (Ok(()), attribute_value_ok),
            (
                Err(ValueError::DoesNotExist(String::from("foo"))),
                attribute_value_does_not_exist,
            ),
            (
                Err(ValueError::Failure(String::from("foo"))),
                attribute_value_failure,
            ),
        ];

        let mut value = Value::Int(0);
        let mut result: Result<(), ValueError>;
        for (expected, case) in cases {
            plugin.vtable.attribute_value = case;
            result = attribute_value(&plugin, 0, &mut value);
            assert_eq!(expected, result);
        }

        tear_down(plugin);
    }

    fn set_up() -> Plugin {
        let peripheral = Box::into_raw(Box::new(MockPeripheral {})) as *mut Peripheral;
        let vtable = VTable {
            peripheral_free: def_peripheral_free,
            error_message: def_error_message,
            attribute_name: def_attribute_name,
            attribute_value: def_attribute_value,
            set_attribute_value: def_set_attribute_value,
        };
        Plugin { peripheral, vtable }
    }

    fn tear_down(plugin: Plugin) {
        unsafe { Box::from_raw(plugin.peripheral) };
    }

    struct MockPeripheral {}

    // Default function pointers for the vtable
    extern "C" fn def_peripheral_free(_: *mut Peripheral) {}

    extern "C" fn def_error_message(_: c_int) -> *const c_uchar {
        b"foo\0" as *const c_uchar
    }

    extern "C" fn def_attribute_name(
        _: *const Peripheral,
        _: size_t,
        _: *mut c_uchar,
        _: size_t,
    ) -> c_int {
        0
    }
    extern "C" fn def_attribute_value(_: *const Peripheral, _: size_t, _: *mut Value) -> c_int {
        0
    }
    extern "C" fn def_set_attribute_value(_: *mut Peripheral, _: size_t, _: *const Value) -> c_int {
        0
    }

    // Function pointers used by different test cases
    extern "C" fn attribute_name_ok(
        _: *const Peripheral,
        _: size_t,
        _: *mut c_uchar,
        _: size_t,
    ) -> c_int {
        PLUGIN_OK
    }
    extern "C" fn attribute_name_does_not_exist(
        _: *const Peripheral,
        _: size_t,
        _: *mut c_uchar,
        _: size_t,
    ) -> c_int {
        ATTRIBUTE_DOES_NOT_EXIST
    }
    extern "C" fn attribute_name_failure(
        _: *const Peripheral,
        _: size_t,
        _: *mut c_uchar,
        _: size_t,
    ) -> c_int {
        999
    }
    extern "C" fn attribute_value_ok(_: *const Peripheral, _: size_t, _: *mut Value) -> c_int {
        PLUGIN_OK
    }
    extern "C" fn attribute_value_does_not_exist(
        _: *const Peripheral,
        _: size_t,
        _: *mut Value,
    ) -> c_int {
        ATTRIBUTE_DOES_NOT_EXIST
    }
    extern "C" fn attribute_value_failure(_: *const Peripheral, _: size_t, _: *mut Value) -> c_int {
        999
    }
}
