use std::ffi::CStr;

use libc::{c_uchar, size_t};
use log;
use memchr::memchr;

use kpal_peripheral::constants::*;
use kpal_peripheral::Value;

use crate::constants::*;
use crate::plugins::Plugin;

pub fn attribute_value(plugin: &Plugin, index: size_t, value: &mut Value) -> ValueResult {
    let result =
        (plugin.vtable.attribute_value)(plugin.object, index as size_t, value as *mut Value);

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

pub fn attribute_name(plugin: &Plugin, index: size_t, name: &mut [u8]) -> NameResult {
    // Reset all bytes to prevent accidental truncation of the name from previous iterations.
    name.iter_mut().for_each(|x| *x = 0);

    let result = (plugin.vtable.attribute_name)(
        plugin.object,
        index as size_t,
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

pub enum ValueResult {
    Success,
    DoesNotExist,
    Failure,
}

pub enum NameResult {
    Success(String),
    DoesNotExist,
    Failure,
}
