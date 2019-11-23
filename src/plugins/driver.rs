//! Methods for communicating directly with Plugins.
use std::ffi::CStr;

use libc::{c_uchar, size_t};
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

    if result == PERIPHERAL_OK {
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
                log::debug!("{}", err);
                String::from("Unknown")
            }
        };

        log::debug!("Received name: {:?}", name);
        Ok(name)
    } else if result == PERIPHERAL_ATTRIBUTE_DOES_NOT_EXIST {
        log::debug!("Attribute does not exist: {}", result);
        Err(NameError::DoesNotExist)
    } else {
        log::debug!(
            "Received error code while getting attribute name: {}",
            result
        );
        Err(NameError::Failure)
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

    if result == PERIPHERAL_OK {
        log::debug!("Received value: {:?}", value);
        Ok(())
    } else if result == PERIPHERAL_ATTRIBUTE_DOES_NOT_EXIST {
        log::debug!("Attribute does not exist: {}", result);
        Err(ValueError::DoesNotExist)
    } else {
        log::debug!(
            "Received error code while fetching attribute value: {}",
            result
        );
        Err(ValueError::Failure)
    }
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

    if result == PERIPHERAL_OK {
        log::debug!("Set value: {:?}", value);
        Ok(())
    } else if result == PERIPHERAL_ATTRIBUTE_DOES_NOT_EXIST {
        log::debug!("Attribute does not exist: {}", result);
        Err(SetValueError::DoesNotExist)
    } else {
        log::debug!(
            "Received error code while setting attribute value: {}",
            result
        );
        Err(SetValueError::Failure)
    }
}

/// Represents the state of a result obtained by fetching a name from an attribute.
#[derive(Debug, PartialEq)]
pub enum NameError {
    DoesNotExist,
    Failure,
}

/// Represents the state of a result obtained by fetching a value from an attribute.
#[derive(Debug, PartialEq)]
pub enum ValueError {
    DoesNotExist,
    Failure,
}

/// Represents the state of a result obtained by setting a value of an attribute.
#[derive(Debug, PartialEq)]
pub enum SetValueError {
    DoesNotExist,
    Failure,
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::boxed::Box;

    use kpal_plugin::{Peripheral, Plugin, VTable, Value};
    use libc::{c_int, c_uchar, size_t};

    use crate::plugins::driver::{NameError, ValueError};

    #[test]
    fn test_attribute_name() {
        let mut plugin = set_up();
        let cases: Vec<(
            Result<String, NameError>,
            extern "C" fn(*const Peripheral, size_t, *mut c_uchar, size_t) -> c_int,
        )> = vec![
            (Ok(String::from("")), attribute_name_ok),
            (Err(NameError::DoesNotExist), attribute_name_does_not_exist),
            (Err(NameError::Failure), attribute_name_failure),
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
                Err(ValueError::DoesNotExist),
                attribute_value_does_not_exist,
            ),
            (Err(ValueError::Failure), attribute_value_failure),
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
        PERIPHERAL_OK
    }
    extern "C" fn attribute_name_does_not_exist(
        _: *const Peripheral,
        _: size_t,
        _: *mut c_uchar,
        _: size_t,
    ) -> c_int {
        PERIPHERAL_ATTRIBUTE_DOES_NOT_EXIST
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
        PERIPHERAL_OK
    }
    extern "C" fn attribute_value_does_not_exist(
        _: *const Peripheral,
        _: size_t,
        _: *mut Value,
    ) -> c_int {
        PERIPHERAL_ATTRIBUTE_DOES_NOT_EXIST
    }
    extern "C" fn attribute_value_failure(_: *const Peripheral, _: size_t, _: *mut Value) -> c_int {
        999
    }
}
