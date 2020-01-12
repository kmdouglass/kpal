//! Executors handle all communication with plugins.

use std::{ffi::CStr, sync::mpsc::channel, thread};

use {
    libc::{c_char, c_int, c_uchar, size_t},
    log,
    memchr::memchr,
};

use kpal_plugin::{constants::*, Val};

use super::{
    errors::{ExecutorError, NameError, PluginError, SetValueError, ValueError},
    messaging::{Receiver, Transmitter},
    Plugin,
};

use crate::{
    constants::*,
    models::{Attribute, Model, Peripheral},
};

/// Executes tasks on a Plugin in response to messages.
///
/// Each Plugin is powered by a single executor.
pub struct Executor {
    /// The Plugin instance that is managed by this executor.
    pub plugin: Plugin,

    /// A copy of a Peripheral model.
    pub peripheral: Peripheral,

    /// The executor's receiver.
    pub rx: Receiver,

    /// The executor's transmitter.
    pub tx: Transmitter,
}

impl Executor {
    /// Returns a new instance of an executor.
    ///
    /// # Arguments
    ///
    /// * `plugin` - The Plugin instance that is managed by this Executor
    /// * `peripheral` - A copy of a Peripheral. This is used to update the corresponding model
    pub fn new(plugin: Plugin, peripheral: Peripheral) -> Executor {
        let (tx, rx) = channel();

        Executor {
            plugin,
            peripheral,
            rx,
            tx,
        }
    }

    /// Starts an Executor.
    ///
    /// The Executor runs inside an infinite loop. During one iteration of the loop, it checks for
    /// a new message in its message queue. If found, it processes the message (possibly by
    /// communicating with the peripheral through the plugin interface) and returns the via the
    /// return transmitter that was passed alongside the message.
    ///
    /// This is a function and not a method of a Executor instance because the function takes
    /// ownership of the instance.
    ///
    /// # Arguments
    ///
    /// - `executor` - An Executor instance. This will be consumed by the function and cannot be
    /// used again after this function is called.
    pub fn run(mut self) {
        thread::spawn(move || -> Result<(), ExecutorError> {
            log::info!("Spawning new thread for plugin: {:?}", self.plugin);

            loop {
                log::debug!(
                    "Checking for messages for peripheral: {}",
                    self.peripheral.id()
                );
                let msg = self.rx.recv().map_err(|_| ExecutorError {})?;
                msg.handle(&mut self);
            }
        });
    }

    /// Returns the name of an attribute from a Plugin.
    ///
    /// # Arguments
    ///
    /// * `id` - The attribute's unique ID
    pub fn attribute_name(&self, id: size_t) -> Result<String, NameError> {
        let mut name = [0u8; ATTRIBUTE_NAME_BUFFER_LENGTH];

        let result = unsafe {
            (self.plugin.vtable.attribute_name)(
                self.plugin.plugin_data,
                id,
                &mut name[0] as *mut c_uchar,
                ATTRIBUTE_NAME_BUFFER_LENGTH,
            )
        };

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
            let msg = unsafe {
                self.error_message(result)
                    .unwrap_or_else(|_| String::from(""))
            };
            Err(NameError::DoesNotExist(msg))
        } else {
            log::error!(
                "Received error code while getting attribute name: {}",
                result
            );
            let msg = unsafe {
                self.error_message(result)
                    .unwrap_or_else(|_| String::from(""))
            };
            Err(NameError::Failure(msg))
        }
    }

    /// Returns the value of an attribute from a Plugin.
    ///
    /// # Arguments
    ///
    /// * `id` - The attribute's unique ID
    /// * `value` - A reference to a value instance into which the attribute's value will be copied
    pub fn attribute_value(&self, id: size_t, value: &mut Val) -> Result<(), ValueError> {
        let result = unsafe {
            (self.plugin.vtable.attribute_value)(self.plugin.plugin_data, id, value as *mut Val)
        };

        if result == PLUGIN_OK {
            log::debug!("Received value: {:?}", value);
            Ok(())
        } else if result == ATTRIBUTE_DOES_NOT_EXIST {
            log::debug!("Attribute does not exist: {}", result);
            let msg = unsafe {
                self.error_message(result)
                    .unwrap_or_else(|_| String::from(""))
            };
            Err(ValueError::DoesNotExist(msg))
        } else {
            log::error!(
                "Received error code while fetching attribute value: {}",
                result
            );
            let msg = unsafe {
                self.error_message(result)
                    .unwrap_or_else(|_| String::from(""))
            };
            Err(ValueError::Failure(msg))
        }
    }

    /// Sets the value of an attribute of a Plugin.
    ///
    /// # Arguments
    ///
    /// * `id` - The attribute's unique ID
    /// * `value` - A reference to a value instance that will be copied into the plugin
    pub fn set_attribute_value(&self, id: size_t, value: &Val) -> Result<(), SetValueError> {
        let result = unsafe {
            (self.plugin.vtable.set_attribute_value)(
                self.plugin.plugin_data,
                id,
                value as *const Val,
            )
        };

        if result == PLUGIN_OK {
            log::debug!("Set value: {:?}", value);
            Ok(())
        } else if result == ATTRIBUTE_DOES_NOT_EXIST {
            log::debug!("Attribute does not exist: {}", result);
            let msg = unsafe {
                self.error_message(result)
                    .unwrap_or_else(|_| String::from(""))
            };
            Err(SetValueError::DoesNotExist(msg))
        } else {
            log::error!(
                "Received error code while setting attribute value: {}",
                result
            );
            let msg = unsafe {
                self.error_message(result)
                    .unwrap_or_else(|_| String::from(""))
            };
            Err(SetValueError::Failure(msg))
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
    /// * `error_code` - The integer code for which the corresponding message will be retrieved.
    unsafe fn error_message(&self, error_code: c_int) -> Result<String, PluginError> {
        let msg_p = (self.plugin.vtable.error_message)(error_code) as *const c_char;

        let msg = if msg_p.is_null() {
            return Err(PluginError {
                body: "An unrecognized error code was provided to the plugin".to_string(),
                http_status_code: 500,
            });
        } else {
            CStr::from_ptr(msg_p).to_str()?.to_owned()
        };

        Ok(msg)
    }

    /// Gets all attribute values and names from a Plugin and updates the corresponding Peripheral.
    ///
    /// This method is only called once to initialize the peripheral.
    pub fn init_attributes(&mut self) {
        log::info!("Getting attributes for peripheral {}", self.peripheral.id());

        let mut value = Val::Int(0);
        let mut index = 0;
        let mut attr: Vec<Attribute> = Vec::new();

        loop {
            match self.attribute_value(index, &mut value) {
                Ok(_) => (),
                Err(err) => match err {
                    ValueError::DoesNotExist(_) => break,
                    ValueError::Failure(_) => {
                        index += 1;
                        continue;
                    }
                },
            };

            let name = match self.attribute_name(index) {
                Ok(name) => name,
                Err(err) => match err {
                    NameError::DoesNotExist(_) => break,
                    NameError::Failure(_) => {
                        index += 1;
                        continue;
                    }
                },
            };

            let new_attr = match Attribute::new(value.clone(), index, name) {
                Ok(new_attr) => new_attr,
                Err(err) => {
                    log::error!("Could not create new attribute: {:?}", err);
                    index += 1;
                    continue;
                }
            };
            attr.push(new_attr);

            index += 1;
        }

        self.peripheral.set_attributes(attr);
        self.peripheral.set_attribute_links();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::boxed::Box;

    use libc::{c_int, c_uchar, size_t};

    use kpal_plugin::{Plugin, PluginData, VTable, Val};

    use crate::models::Peripheral as ModelPeripheral;

    type AttributeName = extern "C" fn(*const PluginData, size_t, *mut c_uchar, size_t) -> c_int;
    type AttributeValue = extern "C" fn(*const PluginData, size_t, *mut Val) -> c_int;

    #[test]
    fn test_error_message() {
        let (plugin, peripheral) = set_up();
        let executor = Executor::new(plugin, peripheral);

        let msg = unsafe { executor.error_message(0) };
        assert_eq!("foo", msg.unwrap());
    }

    #[test]
    fn test_attribute_name() {
        let (mut plugin, peripheral) = set_up();
        let cases: Vec<(Result<String, NameError>, AttributeName)> = vec![
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
        let mut executor: Executor;
        for (expected, case) in cases {
            plugin.vtable.attribute_name = case;
            executor = Executor::new(plugin.clone(), peripheral.clone());

            result = executor.attribute_name(0);
            assert_eq!(expected, result);
        }

        tear_down(plugin);
    }

    #[test]
    fn test_attribute_value() {
        let (mut plugin, peripheral) = set_up();
        let cases: Vec<(Result<(), ValueError>, AttributeValue)> = vec![
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

        let mut executor: Executor;
        let mut value = Val::Int(0);
        let mut result: Result<(), ValueError>;
        for (expected, case) in cases {
            plugin.vtable.attribute_value = case;
            executor = Executor::new(plugin.clone(), peripheral.clone());

            result = executor.attribute_value(0, &mut value);
            assert_eq!(expected, result);
        }

        tear_down(plugin);
    }

    #[test]
    fn test_init_attributes() {
        let (plugin, peripheral) = set_up();
        let mut executor = Executor::new(plugin, peripheral);
        let attribute = Attribute::Int {
            id: 0,
            name: String::from("bar"),
            value: 42,
        };

        assert_eq!(executor.peripheral.attributes().len(), 0);

        executor.init_attributes();

        let attrs = executor.peripheral.attributes();
        assert_eq!(attrs.len(), 1);
        assert_eq!(attribute, attrs[0]);
    }

    fn set_up() -> (Plugin, ModelPeripheral) {
        let plugin_data = Box::into_raw(Box::new(MockPluginData {})) as *mut PluginData;
        let vtable = VTable {
            plugin_free: def_peripheral_free,
            error_message: def_error_message,
            attribute_name: def_attribute_name,
            attribute_value: def_attribute_value,
            set_attribute_value: def_set_attribute_value,
        };
        let plugin = Plugin {
            plugin_data,
            vtable,
        };

        let model: ModelPeripheral =
            serde_json::from_str(r#"{"name":"foo","library_id":0}"#).unwrap();

        (plugin, model)
    }

    fn tear_down(plugin: Plugin) {
        unsafe { Box::from_raw(plugin.plugin_data) };
    }

    struct MockPluginData {}

    // Default function pointers for the vtable
    extern "C" fn def_peripheral_free(_: *mut PluginData) {}

    extern "C" fn def_error_message(_: c_int) -> *const c_uchar {
        b"foo\0" as *const c_uchar
    }

    extern "C" fn def_attribute_name(
        _: *const PluginData,
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
    extern "C" fn def_attribute_value(_: *const PluginData, id: size_t, value: *mut Val) -> c_int {
        if id == 0 {
            unsafe { *value = Val::Int(42) };
            PLUGIN_OK
        } else {
            ATTRIBUTE_DOES_NOT_EXIST
        }
    }
    extern "C" fn def_set_attribute_value(_: *mut PluginData, _: size_t, _: *const Val) -> c_int {
        0
    }

    // Function pointers used by different test cases
    extern "C" fn attribute_name_ok(
        _: *const PluginData,
        _: size_t,
        _: *mut c_uchar,
        _: size_t,
    ) -> c_int {
        PLUGIN_OK
    }
    extern "C" fn attribute_name_does_not_exist(
        _: *const PluginData,
        _: size_t,
        _: *mut c_uchar,
        _: size_t,
    ) -> c_int {
        ATTRIBUTE_DOES_NOT_EXIST
    }
    extern "C" fn attribute_name_failure(
        _: *const PluginData,
        _: size_t,
        _: *mut c_uchar,
        _: size_t,
    ) -> c_int {
        999
    }
    extern "C" fn attribute_value_ok(_: *const PluginData, _: size_t, _: *mut Val) -> c_int {
        PLUGIN_OK
    }
    extern "C" fn attribute_value_does_not_exist(
        _: *const PluginData,
        _: size_t,
        _: *mut Val,
    ) -> c_int {
        ATTRIBUTE_DOES_NOT_EXIST
    }
    extern "C" fn attribute_value_failure(_: *const PluginData, _: size_t, _: *mut Val) -> c_int {
        999
    }
}
