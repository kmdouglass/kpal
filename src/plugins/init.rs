//! Initialization routines for a Plugin.
use log;

use kpal_plugin::Value;

use super::driver::{attribute_name, attribute_value, NameError, ValueError};
use super::Plugin;

use crate::models::Model;
use crate::models::{Attribute, Peripheral};

/// Gets all attribute values and names from a Plugin and updates the corresponding Peripheral.
///
/// # Arguments
///
/// * `peripheral` - The Peripheral instance to update
/// * `plugin` - The plugin whose attributes will be fetched
pub fn attributes(peripheral: &mut Peripheral, plugin: &Plugin) {
    log::info!("Getting attributes for peripheral {}", peripheral.id());

    let mut value = Value::Int(0);
    let mut index = 0;
    let mut attr: Vec<Attribute> = Vec::new();

    loop {
        match attribute_value(plugin, index, &mut value) {
            Ok(_) => (),
            Err(err) => match err {
                ValueError::DoesNotExist(_) => break,
                ValueError::Failure(_) => {
                    index += 1;
                    continue;
                }
            },
        };

        let name = match attribute_name(plugin, index) {
            Ok(name) => name,
            Err(err) => match err {
                NameError::DoesNotExist(_) => break,
                NameError::Failure(_) => {
                    index += 1;
                    continue;
                }
            },
        };

        attr.push(Attribute::new(value.clone(), index, name));

        index += 1;
    }

    peripheral.set_attributes(attr);
    peripheral.set_attribute_links();
}

#[cfg(test)]
mod tests {
    use super::*;

    use kpal_plugin::constants::*;
    use kpal_plugin::{Peripheral, Plugin, VTable, Value};
    use libc::{c_int, c_uchar, size_t};
    use serde_json;

    use crate::constants::ATTRIBUTE_NAME_BUFFER_LENGTH;
    use crate::models::Peripheral as ModelPeripheral;

    #[test]
    fn test_attributes() {
        let mut context = set_up();

        assert_eq!(context.model_peripheral.attributes().len(), 0);

        attributes(&mut context.model_peripheral, &context.plugin);

        let attrs = context.model_peripheral.attributes();
        assert_eq!(attrs.len(), 1);
        assert_eq!(context.attribute, attrs[0]);

        tear_down(context.plugin);
    }

    struct Context {
        attribute: Attribute,
        model_peripheral: ModelPeripheral,
        plugin: Plugin,
    }

    fn set_up() -> Context {
        let peripheral = Box::into_raw(Box::new(MockPeripheral {})) as *mut Peripheral;
        let model_peripheral = String::from("{\"name\":\"foo\", \"library_id\":0}");
        let model_peripheral: ModelPeripheral = serde_json::from_str(&model_peripheral)
            .expect("Could not create peripheral from JSON string");

        let vtable = VTable {
            peripheral_free: def_peripheral_free,
            error_message: def_error_message,
            attribute_name: def_attribute_name,
            attribute_value: def_attribute_value,
            set_attribute_value: def_set_attribute_value,
        };

        let plugin = Plugin { peripheral, vtable };

        Context {
            attribute: Attribute::Int {
                id: 0,
                name: String::from("bar"),
                value: 42,
            },
            model_peripheral,
            plugin,
        }
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
        id: size_t,
        buffer: *mut c_uchar,
        _: size_t,
    ) -> c_int {
        if id == 0 {
            unsafe {
                let string: &[u8] = b"bar\0";
                let buffer = std::slice::from_raw_parts_mut(buffer, ATTRIBUTE_NAME_BUFFER_LENGTH);
                buffer[0..4].copy_from_slice(string);
            };
            PLUGIN_OK
        } else {
            ATTRIBUTE_DOES_NOT_EXIST
        }
    }
    extern "C" fn def_attribute_value(
        _: *const Peripheral,
        id: size_t,
        value: *mut Value,
    ) -> c_int {
        if id == 0 {
            unsafe { *value = Value::Int(42) };
            PLUGIN_OK
        } else {
            ATTRIBUTE_DOES_NOT_EXIST
        }
    }
    extern "C" fn def_set_attribute_value(_: *mut Peripheral, _: size_t, _: *const Value) -> c_int {
        0
    }
}
