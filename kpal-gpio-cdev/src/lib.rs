//! KPAL plugin to control the output of a single GPIO pin using the GPIO char device.
mod errors;

use std::{cell::RefCell, convert::TryInto, ffi::CString};

use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use libc::c_int;
use log;

use kpal_plugin::{error_codes::*, *};

use crate::errors::*;

const DEFAULT_DEVICE_FILE: &str = "/dev/gpiochip0";

/// The GPIO pin number.
const DEFAULT_OFFSET: u32 = 4;

/// Holds the state of the plugin, including the chip and line handles.
#[derive(Debug)]
#[repr(C)]
struct GPIOPlugin {
    /// The collection of attributes that describe this plugin.
    attributes: Attributes<Self, GPIOPluginError>,

    /// A handle to the chip that represents the character device.
    chip: Option<RefCell<Chip>>,

    /// A handle to the particular GPIO line that is controlled by this plugin.
    line_handle: Option<LineHandle>,
}

impl PluginAPI<GPIOPluginError> for GPIOPlugin {
    /// Returns a new instance of a GPIO plugin.
    fn new() -> Result<GPIOPlugin, GPIOPluginError> {
        let attributes = RefCell::new(multimap! {
            0, "device file" => Attribute {
                    name: CString::new("Device file").unwrap(),
                    value: Value::String(CString::new(DEFAULT_DEVICE_FILE).unwrap()),
                    callbacks_init: Callbacks::Update,
                    callbacks_run: Callbacks::Constant,
            },
            1, "offset" => Attribute {
                    name: CString::new("Offset").unwrap(),
                    value: Value::Uint(DEFAULT_OFFSET),
                    callbacks_init: Callbacks::Update,
                    callbacks_run: Callbacks::Constant,
            },
            2, "pin state" => Attribute {
                    name: CString::new("Pin state").unwrap(),
                    value: Value::Int(0),
                    callbacks_init: Callbacks::Constant,
                    callbacks_run: Callbacks::GetAndSet(on_get_pin_state, on_set_pin_state),
            },
        });

        Ok(GPIOPlugin {
            attributes,
            chip: None,
            line_handle: None,
        })
    }

    /// Initializes the GPIO hardware device.
    fn init(&mut self) -> Result<(), GPIOPluginError> {
        let device_file = if let Value::String(device_file) = &self
            .attributes
            .borrow()
            .get_alt(&"device file")
            .ok_or(GPIOPluginError {
                error_code: ATTRIBUTE_DOES_NOT_EXIST,
                side: None,
            })?
            .value
        {
            device_file.clone().into_string()?
        } else {
            unreachable!()
        };
        let mut chip = Chip::new(device_file)?;

        let offset = if let Value::Uint(offset) = self
            .attributes
            .borrow()
            .get_alt(&"offset")
            .ok_or(GPIOPluginError {
                error_code: ATTRIBUTE_DOES_NOT_EXIST,
                side: None,
            })?
            .value
        {
            offset
        } else {
            unreachable!()
        };

        let handle = chip
            .get_line(offset)?
            .request(LineRequestFlags::OUTPUT, 0, "set-output")?;

        self.chip = Some(RefCell::new(chip));
        self.line_handle = Some(handle);

        Ok(())
    }

    fn attributes(&self) -> &Attributes<GPIOPlugin, GPIOPluginError> {
        &self.attributes
    }
}

/// The callback function that is fired when the pin state is read during the run phase.
///
/// # Arguments
///
/// * `plugin` - A reference to the struct that contains the plugin's state.
/// * `_cached` - The most recently read or modified value of the attribute.
fn on_get_pin_state(plugin: &GPIOPlugin, _cached: &Value) -> Result<Value, GPIOPluginError> {
    let pin_value = plugin
        .line_handle
        .as_ref()
        .ok_or_else(|| PluginUninitializedError {})?
        .get_value()?;
    let value = Value::Int(pin_value.try_into()?);

    Ok(value)
}

/// The callback function that is fired when the pin state is set.
///
/// # Arguments
///
/// * `plugin` - A reference to the struct that contains the plugin's state.
/// * `_cached` - The most recently read or modified value of the attribute.
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

    plugin
        .line_handle
        .as_ref()
        .ok_or_else(|| PluginUninitializedError {})?
        .set_value(pin_value)?;

    Ok(())
}

declare_plugin!(GPIOPlugin, GPIOPluginError);
