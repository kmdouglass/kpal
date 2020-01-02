//! KPAL plugin to control the output of a single GPIO pin using the GPIO char device.
mod errors;

use std::{cell::RefCell, convert::TryInto, ffi::CString};

use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use libc::c_int;
use log;

use kpal_plugin::{constants::*, *};

use crate::errors::*;

const DEVICE_FILE: &str = "/dev/gpiochip0";

/// The GPIO pin number.
const OFFSET: u32 = 4;

/// Holds the state of the plugin, including the chip and line handles.
#[derive(Debug)]
#[repr(C)]
struct GPIOPlugin {
    /// The collection of attributes that describe this plugin.
    attributes: Attributes<Self, GPIOPluginError>,

    /// A handle to the chip that represents the character device.
    chip: RefCell<Chip>,

    /// A handle to the particular GPIO line that is controlled by this plugin.
    line_handle: LineHandle,
}

impl PluginAPI<GPIOPluginError> for GPIOPlugin {
    /// Returns a new instance of a GPIO plugin.
    fn new() -> Result<GPIOPlugin, GPIOPluginError> {
        let mut chip = Chip::new(DEVICE_FILE)?;

        let handle = chip
            .get_line(OFFSET)?
            .request(LineRequestFlags::OUTPUT, 0, "set-output")?;

        let attributes = RefCell::new(vec![Attribute {
            name: CString::new("Pin state").unwrap(),
            value: Value::Int(0),
            callbacks: Callbacks::GetAndSet(on_get_pin_state, on_set_pin_state),
        }]);

        Ok(GPIOPlugin {
            attributes,
            chip: RefCell::new(chip),
            line_handle: handle,
        })
    }

    fn attributes(&self) -> &Attributes<GPIOPlugin, GPIOPluginError> {
        &self.attributes
    }
}

/// The callback function that is fired when the pin state is read.
///
/// # Arguments
///
/// * `plugin` - A reference to the struct that contains the plugin's state.
/// * `cached` - The most recently read or modified value of the attribute.
fn on_get_pin_state(plugin: &GPIOPlugin, _cached: &Value) -> Result<Value, GPIOPluginError> {
    let pin_value = plugin.line_handle.get_value()?;
    let value = Value::Int(pin_value.try_into()?);

    Ok(value)
}

/// The callback function that is fired when the pin state is set.
///
/// # Arguments
///
/// * `plugin` - A reference to the struct that contains the plugin's state.
/// * `cached` - The most recently read or modified value of the attribute.
/// * `val` -  The new value of the attribute.
fn on_set_pin_state(
    plugin: &GPIOPlugin,
    _cached: &Value,
    val: &Val,
) -> Result<(), GPIOPluginError> {
    let pin_value = if let Val::Int(pin_value) = val {
        pin_value.to_owned().try_into()?
    } else {
        unreachable!()
    };

    plugin.line_handle.set_value(pin_value)?;

    Ok(())
}

declare_plugin!(GPIOPlugin, GPIOPluginError);
