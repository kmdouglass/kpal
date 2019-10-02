//! Methods for communicating directly with Plugins.
use std::ffi::CStr;

use libc::{c_uchar, size_t};
use log;
use memchr::memchr;

use kpal_plugin::constants::*;
use kpal_plugin::Value;

use crate::constants::*;
use crate::plugins::Plugin;

/// Returns the value of an attribute from a Plugin.
///
/// # Arguments
///
/// * `plugin` - A reference to the Plugin from which an attribute will be obtained
/// * `id` - The attribute's unique ID
/// * `value` - A reference to a value instance into which the attribute's value will be copied
pub fn attribute_value(plugin: &Plugin, id: size_t, value: &mut Value) -> ValueResult {
    let result =
        (plugin.vtable.attribute_value)(plugin.peripheral, id as size_t, value as *mut Value);

    if result == PERIPHERAL_OK {
        log::debug!("Received value: {:?}", value);
        ValueResult::Success
    } else if result == PERIPHERAL_ATTRIBUTE_DOES_NOT_EXIST {
        log::debug!("Attribute does not exist: {}", result);
        ValueResult::DoesNotExist
    } else {
        log::debug!(
            "Received error code while fetching attribute value: {}",
            result
        );
        ValueResult::Failure
    }
}

/// Returns the name of an attribute from a Plugin.
///
/// # Arguments
///
/// * `plugin` - A reference to the Plugin from which an attribute's name will be obtained
/// * `id` - The attribute's unique ID
/// * `name` - A buffer into which the attribute's name will be copied
pub fn attribute_name(plugin: &Plugin, id: size_t, name: &mut [u8]) -> NameResult {
    // Reset all bytes to prevent accidental truncation of the name from previous iterations.
    name.iter_mut().for_each(|x| *x = 0);

    let result = (plugin.vtable.attribute_name)(
        plugin.peripheral,
        id as size_t,
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
        NameResult::Success(name)
    } else if result == PERIPHERAL_ATTRIBUTE_DOES_NOT_EXIST {
        log::debug!("Attribute does not exist: {}", result);
        NameResult::DoesNotExist
    } else {
        log::debug!(
            "Received error code while getting attribute name: {}",
            result
        );
        NameResult::Failure
    }
}

/// Represents the state of a result obtained by fetching a value from an attribute.
pub enum ValueResult {
    Success,
    DoesNotExist,
    Failure,
}

/// Represents the state of a result obtained by fetching a name from an attribute.
pub enum NameResult {
    Success(String),
    DoesNotExist,
    Failure,
}
