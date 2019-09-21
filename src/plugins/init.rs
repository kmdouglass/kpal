use std::ffi::CStr;

use libc::{c_uchar, size_t};
use log;
use memchr::memchr;

use kpal_peripheral::constants::*;
use kpal_peripheral::Value;

use crate::constants::*;
use crate::models::database::Query;
use crate::models::{Attribute, Peripheral};
use crate::plugins::Plugin;

pub fn fetch_attributes(peripheral: &mut Peripheral, plugin: &Plugin) {
    log::info!("Fetching attributes for peripheral {}", peripheral.id());

    let mut value = Value::Int(0);
    let mut name = [0u8; ATTRIBUTE_NAME_BUFFER_LENGTH];
    let mut index = 0;
    let mut attr: Vec<Attribute> = Vec::new();

    loop {
        match fetch_attribute_value(peripheral, plugin, index, &mut value) {
            ValueResult::Success => (),
            ValueResult::DoesNotExist => break,
            ValueResult::Failure => {
                index += 1;
                continue;
            }
        };

        let name = match fetch_attribute_name(peripheral, plugin, index, &mut name) {
            NameResult::Success(name) => name,
            NameResult::DoesNotExist => break,
            NameResult::Failure => {
                index += 1;
                continue;
            }
        };

        attr.push(match value {
            Value::Int(value) => Attribute::Int {
                id: index,
                name: name,
                value: value,
            },
            Value::Float(value) => Attribute::Float {
                id: index,
                name: name,
                value: value,
            },
        });

        index += 1;
    }

    peripheral.set_attributes(attr);
}

fn fetch_attribute_value(
    peripheral: &mut Peripheral,
    plugin: &Plugin,
    index: size_t,
    value: &mut Value,
) -> ValueResult {
    let result =
        (plugin.vtable.attribute_value)(plugin.object, index as size_t, value as *mut Value);

    if result == PERIPHERAL_OK {
        log::debug!(
            "Received value {:?} from peripheral {}",
            value,
            peripheral.id()
        );
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

fn fetch_attribute_name(
    peripheral: &mut Peripheral,
    plugin: &Plugin,
    index: size_t,
    name: &mut [u8],
) -> NameResult {
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

        log::debug!(
            "Received name {:?} from peripheral {}",
            name,
            peripheral.id()
        );

        NameResult::Success(name)
    } else if result == PERIPHERAL_ATTRIBUTE_DOES_NOT_EXIST {
        log::debug!("Attribute does not exist: {}", result);
        NameResult::DoesNotExist
    } else {
        log::debug!(
            "Received error code while fetching attribute name: {}",
            result
        );
        NameResult::Failure
    }
}

enum ValueResult {
    Success,
    DoesNotExist,
    Failure,
}

enum NameResult {
    Success(String),
    DoesNotExist,
    Failure,
}
