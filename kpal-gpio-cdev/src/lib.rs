//! KPAL plugin to control the output of a single GPIO pin using the GPIO char device.
mod errors;

use std::cell::RefCell;
use std::convert::TryInto;
use std::ffi::{CStr, CString};

use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use libc::c_int;
use log;

use kpal_plugin::constants::*;
use kpal_plugin::*;

use crate::errors::*;

const DEVICE_FILE: &str = "/dev/gpiochip0";

/// The GPIO pin number.
const OFFSET: u32 = 4;

#[derive(Debug)]
#[repr(C)]
struct GPIOPlugin {
    chip: RefCell<Chip>,
    line_handle: LineHandle,
    pin_state_label: CString,
}

impl PluginAPI<GPIOPluginError> for GPIOPlugin {
    type Plugin = GPIOPlugin;

    /// Returns a new instance of a GPIO plugin.
    fn new() -> Result<GPIOPlugin, GPIOPluginError> {
        let mut chip = Chip::new(DEVICE_FILE)?;

        let handle = chip
            .get_line(OFFSET)?
            .request(LineRequestFlags::OUTPUT, 0, "set-output")?;

        Ok(GPIOPlugin {
            chip: RefCell::new(chip),
            line_handle: handle,
            pin_state_label: CString::new("Pin state").expect("failed to create attribute name"),
        })
    }

    fn attribute_name(&self, id: usize) -> Result<&CStr, GPIOPluginError> {
        log::debug!("Received request for the name of attribute: {}", id);
        match id {
            0 => Ok(&self.pin_state_label),
            _ => Err(GPIOPluginError {
                error_code: ATTRIBUTE_DOES_NOT_EXIST,
                side: None,
            }),
        }
    }

    fn attribute_value(&self, id: usize) -> Result<Value, GPIOPluginError> {
        log::debug!("Received request for the value of attribute: {}", id);
        let value = self.line_handle.get_value()?;

        let value = value.try_into()?;

        Ok(Value::Int(value))
    }

    fn attribute_set_value(&mut self, id: usize, value: &Value) -> Result<(), GPIOPluginError> {
        log::debug!("Received request to set the value of attribute: {}", id);
        let value = match value {
            Value::Int(value) => value.to_owned(),
            _ => {
                return Err(GPIOPluginError {
                    error_code: ATTRIBUTE_TYPE_MISMATCH,
                    side: None,
                })
            }
        }
        .try_into()?;

        self.line_handle.set_value(value)?;

        Ok(())
    }
}

// FFI
#[no_mangle]
pub extern "C" fn kpal_library_init() -> c_int {
    env_logger::init();
    PLUGIN_OK
}

/// Initialize the plugin.
///
/// # Safety
///
/// This function is unsafe because it dereferences a null pointer and assigns data to a variable
/// of the type `MaybeUnit`.
#[no_mangle]
pub unsafe extern "C" fn kpal_plugin_init(plugin: *mut Plugin) -> c_int {
    let plugin_data = match GPIOPlugin::new() {
        Ok(plugin_data) => plugin_data,
        Err(e) => {
            log::error!("Failed to initialize the plugin: {:?}", e);
            return PLUGIN_INIT_ERR;
        }
    };

    let plugin_data: Box<GPIOPlugin> = Box::new(plugin_data);
    let plugin_data = Box::into_raw(plugin_data) as *mut Peripheral;

    let vtable = VTable {
        peripheral_free,
        error_message,
        attribute_name: attribute_name::<GPIOPlugin, GPIOPluginError>,
        attribute_value: attribute_value::<GPIOPlugin, GPIOPluginError>,
        set_attribute_value: set_attribute_value::<GPIOPlugin, GPIOPluginError>,
    };

    plugin.write(Plugin {
        peripheral: plugin_data,
        vtable,
    });

    log::debug!("Initialized plugin: {:?}", plugin);
    PLUGIN_OK
}
