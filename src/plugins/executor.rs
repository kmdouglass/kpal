use std::{error::Error, ffi::CStr, fmt, sync::mpsc::channel, thread};

use {
    libc::{c_char, c_int, c_uchar, size_t},
    log,
    memchr::memchr,
};

use kpal_plugin::{constants::*, Value};

use super::messaging::{Receiver, Transmitter};
use super::Plugin;

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
        thread::spawn(move || -> Result<(), ExecutorRuntimeError> {
            log::info!("Spawning new thread for plugin: {:?}", self.plugin);

            loop {
                log::debug!(
                    "Checking for messages for peripheral: {}",
                    self.peripheral.id()
                );
                let msg = self.rx.recv().map_err(|_| ExecutorRuntimeError {})?;
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
                self.plugin.peripheral,
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
    pub fn attribute_value(&self, id: size_t, value: &mut Value) -> Result<(), ValueError> {
        let result = unsafe {
            (self.plugin.vtable.attribute_value)(self.plugin.peripheral, id, value as *mut Value)
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
    pub fn set_attribute_value(&self, id: size_t, value: &Value) -> Result<(), SetValueError> {
        let result = unsafe {
            (self.plugin.vtable.set_attribute_value)(
                self.plugin.peripheral,
                id,
                value as *const Value,
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
    /// * `lib` - A copy of the Library that contains the implementation of the peripheral's Plugin API
    unsafe fn error_message(&self, error_code: c_int) -> Result<String, KpalErrorMsg> {
        let msg_p = (self.plugin.vtable.error_message)(error_code) as *const c_char;

        let msg = if msg_p.is_null() {
            return Err(KpalErrorMsg {});
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

        let mut value = Value::Int(0);
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

            attr.push(Attribute::new(value.clone(), index, name));

            index += 1;
        }

        self.peripheral.set_attributes(attr);
        self.peripheral.set_attribute_links();
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

/// An error returned by a failed Executor thread.
#[derive(Debug)]
pub struct ExecutorRuntimeError {}

impl Error for ExecutorRuntimeError {
    fn description(&self) -> &str {
        "The executor thread failed"
    }
}

impl fmt::Display for ExecutorRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The executor thread failed")
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::boxed::Box;

    use libc::{c_int, c_uchar, size_t};

    use kpal_plugin::{Peripheral, Plugin, VTable, Value};

    use crate::models::Peripheral as ModelPeripheral;

    type AttributeName = extern "C" fn(*const Peripheral, size_t, *mut c_uchar, size_t) -> c_int;
    type AttributeValue = extern "C" fn(*const Peripheral, size_t, *mut Value) -> c_int;

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
        let mut value = Value::Int(0);
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
        let peripheral = Box::into_raw(Box::new(MockPeripheralData {})) as *mut Peripheral;
        let vtable = VTable {
            peripheral_free: def_peripheral_free,
            error_message: def_error_message,
            attribute_name: def_attribute_name,
            attribute_value: def_attribute_value,
            set_attribute_value: def_set_attribute_value,
        };
        let plugin = Plugin { peripheral, vtable };

        let model: ModelPeripheral =
            serde_json::from_str(r#"{"name":"foo","library_id":0}"#).unwrap();

        (plugin, model)
    }

    fn tear_down(plugin: Plugin) {
        unsafe { Box::from_raw(plugin.peripheral) };
    }

    struct MockPeripheralData {}

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
