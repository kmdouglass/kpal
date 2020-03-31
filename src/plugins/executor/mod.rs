//! Executors handle all communication with plugins.

use std::{collections::BTreeMap, ffi::CStr, sync::mpsc::channel, thread};

use {
    libc::{c_char, c_int, c_uchar, size_t},
    log,
    memchr::memchr,
};

use kpal_plugin::{error_codes::*, Val};
use kpal_plugin::{ATTRIBUTE_PRE_INIT_FALSE, ATTRIBUTE_PRE_INIT_TRUE, INIT_PHASE, RUN_PHASE};

use super::{
    messaging::{Receiver, Transmitter},
    Plugin, PluginError,
};

use crate::{
    constants::*,
    models::{Attribute, Model, Peripheral, PeripheralBuilder},
};

/// Executes tasks on a Plugin in response to messages.
///
/// Each Plugin is powered by a single executor.
pub struct Executor {
    /// The Plugin instance that is managed by this executor.
    pub plugin: Plugin,

    /// The executor's receiver.
    pub rx: Receiver,

    /// The executor's transmitter.
    pub tx: Transmitter,

    /// The current phase of the plugin's lifetime
    phase: i32,
}

impl Executor {
    /// Returns a new instance of an executor.
    ///
    /// # Arguments
    ///
    /// * `plugin` - The Plugin instance that is managed by this Executor
    pub fn new(plugin: Plugin) -> Executor {
        let (tx, rx) = channel();
        let phase = INIT_PHASE;

        Executor {
            plugin,
            rx,
            tx,
            phase,
        }
    }

    /// Starts an Executor.
    ///
    /// The Executor runs inside an infinite loop. During one iteration of the loop, it checks for
    /// a new message in its message queue. If found, it processes the message (possibly by
    /// communicating with the peripheral through the plugin interface) and returns the result via
    /// the return transmitter that was passed alongside the message.
    ///
    /// # Arguments
    ///
    /// * `peripheral` - The instance of a peripheral model that is modified in response to actions
    /// performed on its plugin. Representations of this peripheral are returned to the user upon
    /// request, which allows her/him to query the state of the plugin.
    pub fn run(mut self, mut peripheral: Peripheral) {
        thread::spawn(move || -> Result<(), PluginError> {
            log::info!("Spawning new thread for plugin: {:?}", self.plugin);

            loop {
                log::debug!("Checking for messages for plugin: {}", peripheral.id());
                let msg = self.rx.recv()?;
                msg.handle(&mut self, &mut peripheral);
            }
        });
    }

    /// Returns the number of attributes of a Plugin.
    pub fn attribute_count(&self) -> Result<usize, PluginError> {
        let mut count: usize = 0;
        let result = unsafe {
            (self.plugin.vtable.attribute_count)(self.plugin.plugin_data, &mut count as *mut size_t)
        };

        if result == PLUGIN_OK {
            Ok(count)
        } else {
            Err(PluginError::AttributeCountError)
        }
    }

    /// Returns the set of attribute IDs of a Plugin.
    pub fn attribute_ids(&self) -> Result<Vec<usize>, PluginError> {
        let num_attributes = self.attribute_count()?;
        let mut ids = vec![0usize; num_attributes];

        let result = unsafe {
            (self.plugin.vtable.attribute_ids)(self.plugin.plugin_data, ids.as_mut_ptr(), ids.len())
        };

        if result == PLUGIN_OK {
            Ok(ids)
        } else {
            Err(PluginError::AttributeIDsError)
        }
    }

    /// Returns the name of an attribute from a Plugin.
    ///
    /// # Arguments
    ///
    /// * `id` - The attribute's unique ID
    pub fn attribute_name(&self, id: size_t) -> Result<String, PluginError> {
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
            Err(PluginError::AttributeDoesNotExist(msg))
        } else {
            log::error!(
                "Received error code while getting attribute name: {}",
                result
            );
            let msg = unsafe {
                self.error_message(result)
                    .unwrap_or_else(|_| String::from(""))
            };
            Err(PluginError::AttributeFailure(msg))
        }
    }

    /// Determines whether an attribute may be set before initialization.
    ///
    /// # Arguments
    ///
    /// * `id` - The attribute's unique ID
    pub fn attribute_pre_init(&self, id: size_t) -> Result<bool, PluginError> {
        let mut pre_init: c_char = 0;

        let result = unsafe {
            (self.plugin.vtable.attribute_pre_init)(
                self.plugin.plugin_data,
                id,
                &mut pre_init as *mut c_char,
            )
        };

        if result == PLUGIN_OK {
            log::debug!("Received pre-init status: {}", pre_init);
            if pre_init == ATTRIBUTE_PRE_INIT_TRUE {
                Ok(true)
            } else if pre_init == ATTRIBUTE_PRE_INIT_FALSE {
                Ok(false)
            } else {
                Err(PluginError::AttributeFailure(
                    "could not determine pre-init status from the plugin".to_string(),
                ))
            }
        } else if result == ATTRIBUTE_DOES_NOT_EXIST {
            log::debug!("Attribute does not exist: {}", result);
            let msg = unsafe {
                self.error_message(result).unwrap_or_else(|_| {
                    String::from("could not determine error message from plugin")
                })
            };
            Err(PluginError::AttributeDoesNotExist(msg))
        } else {
            log::error!(
                "Received error code while determining whether the attribute is pre-init: {}",
                result
            );
            let msg = unsafe {
                self.error_message(result).unwrap_or_else(|_| {
                    String::from("could not determine error message from plugin")
                })
            };
            Err(PluginError::AttributeFailure(msg))
        }
    }

    /// Returns the value of an attribute from a Plugin.
    ///
    /// # Arguments
    ///
    /// * `id` - The attribute's unique ID
    /// * `value` - A reference to a value instance into which the attribute's value will be copied
    pub fn attribute_value(&self, id: size_t, value: &mut Val) -> Result<(), PluginError> {
        let result = unsafe {
            (self.plugin.vtable.attribute_value)(
                self.plugin.plugin_data,
                id,
                value as *mut Val,
                self.phase,
            )
        };

        if result == PLUGIN_OK {
            log::debug!("Received value: {:?}", value);
            Ok(())
        } else if result == ATTRIBUTE_DOES_NOT_EXIST {
            log::debug!("Attribute does not exist: {}", result);
            let msg = unsafe {
                self.error_message(result)
                    .unwrap_or_else(|_| String::from("could not get error message from plugin"))
            };
            Err(PluginError::AttributeDoesNotExist(msg))
        } else {
            log::error!(
                "Received error code while fetching attribute value: {}",
                result
            );
            let msg = unsafe {
                self.error_message(result)
                    .unwrap_or_else(|_| String::from("could not get error message from plugin"))
            };
            Err(PluginError::AttributeFailure(msg))
        }
    }

    /// Sets the value of an attribute of a Plugin.
    ///
    /// # Arguments
    ///
    /// * `id` - The attribute's unique ID
    /// * `value` - A reference to a value instance that will be copied into the plugin
    pub fn set_attribute_value(&self, id: size_t, value: &Val) -> Result<(), PluginError> {
        let result = unsafe {
            (self.plugin.vtable.set_attribute_value)(
                self.plugin.plugin_data,
                id,
                value as *const Val,
                self.phase,
            )
        };

        if result == PLUGIN_OK {
            log::debug!("Set value: {:?}", value);
            Ok(())
        } else if result == ATTRIBUTE_DOES_NOT_EXIST {
            log::debug!("Attribute does not exist: {}", id);
            let msg = unsafe {
                self.error_message(result)
                    .unwrap_or_else(|_| String::from("could not get error message from plugin"))
            };
            Err(PluginError::AttributeDoesNotExist(msg))
        } else if result == ATTRIBUTE_IS_NOT_SETTABLE {
            log::debug!("Attribute is not settable: {}", id);
            let msg = unsafe {
                self.error_message(result)
                    .unwrap_or_else(|_| String::from("could not get error message from plugin"))
            };
            Err(PluginError::AttributeNotSettable(msg))
        } else {
            log::error!(
                "Received error code while setting attribute value: {}",
                result
            );
            let msg = unsafe {
                self.error_message(result)
                    .unwrap_or_else(|_| String::from("could not get error message from plugin"))
            };
            Err(PluginError::AttributeFailure(msg))
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
        let msg_p = (self.plugin.vtable.error_message_ns)(error_code) as *const c_char;

        let msg = if msg_p.is_null() {
            return Err(PluginError::MessageNullPointerError);
        } else {
            CStr::from_ptr(msg_p).to_str()?.to_owned()
        };

        Ok(msg)
    }

    /// Advances the plugin to the next lifecycle phase.
    pub fn advance(&mut self) -> Result<i32, PluginError> {
        if self.phase == INIT_PHASE {
            self.phase = RUN_PHASE;
            return Ok(self.phase);
        }

        Err(PluginError::AdvancePhaseError(self.phase))
    }

    /// Gets all attribute values and names from a Plugin and updates the corresponding Peripheral.
    ///
    /// This method is only called once to discover the attributes of the plugin.
    pub fn discover_attributes(&mut self) -> Option<BTreeMap<usize, Attribute>> {
        let ids = match self.attribute_ids() {
            Ok(ids) => ids,
            Err(e) => {
                log::error!("Could not discover plugin attributes: {:?}", e);
                return None;
            }
        };

        let mut value = Val::Int(0);
        let mut attrs: BTreeMap<usize, Attribute> = BTreeMap::new();
        for id in ids {
            match self.attribute_value(id, &mut value) {
                Ok(_) => (),
                Err(err) => {
                    log::error!("Could not discover value of attribute {}: {:?}", id, err);
                    continue;
                }
            };

            let name = match self.attribute_name(id) {
                Ok(name) => name,
                Err(err) => {
                    log::error!("Could not discover name of attribute {}: {:?}", id, err);
                    continue;
                }
            };

            let pre_init = match self.attribute_pre_init(id) {
                Ok(pre_init) => pre_init,
                Err(err) => {
                    log::error!(
                        "Could not discover pre_init status of attribute {}: {:?}",
                        id,
                        err
                    );
                    continue;
                }
            };

            let new_attr = match Attribute::new(value.clone(), id, name, pre_init) {
                Ok(new_attr) => new_attr,
                Err(err) => {
                    log::error!("Could not create new attribute: {:?}", err);
                    continue;
                }
            };
            attrs.insert(id, new_attr);
        }

        if attrs.is_empty() {
            None
        } else {
            Some(attrs)
        }
    }

    /// Initializes the plugin.
    pub fn init(&self) -> Result<(), PluginError> {
        let result = unsafe { (self.plugin.vtable.plugin_init)(self.plugin.plugin_data) };

        if result == PLUGIN_OK {
            log::debug!("Plugin's initialzation routine ran successfully");
            Ok(())
        } else {
            log::error!(
                "Received error code while initialzing the plugin: {}",
                result
            );
            let msg = unsafe {
                self.error_message(result)
                    .unwrap_or_else(|_| String::from("could not get error message from plugin"))
            };
            Err(PluginError::PluginInitError(msg))
        }
    }

    /// Synchronizes the plugin with the peripheral by setting all settable plugin attributes.
    ///
    /// # Arguments
    ///
    /// * `builder` - A reference to peripheral data
    pub fn sync(&mut self, builder: &PeripheralBuilder) -> Result<(), PluginError> {
        for attr in builder.attributes().values() {
            let value = attr.to_value()?;
            let val = value.as_val();

            if let Err(err) = self.set_attribute_value(attr.id(), &val) {
                match err {
                    PluginError::AttributeNotSettable(_) => {
                        log::debug!("Skipping synchronization of attribute: {}", attr.id());
                        continue;
                    }
                    _ => return Err(err),
                }
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::boxed::Box;

    use libc::{c_int, c_uchar, size_t};

    use kpal_plugin::{Phase, Plugin, PluginData, VTable, Val};

    use crate::models::{Peripheral, PeripheralBuilder};

    type AttributeName = extern "C" fn(*const PluginData, size_t, *mut c_uchar, size_t) -> c_int;
    type AttributeValue = extern "C" fn(*const PluginData, size_t, *mut Val, Phase) -> c_int;

    #[test]
    fn test_advance() {
        let (plugin, _) = set_up();
        let mut executor = Executor::new(plugin);

        assert_eq!(INIT_PHASE, executor.phase);

        let mut result = executor.advance();
        assert!(result.is_ok());
        assert_eq!(RUN_PHASE, executor.phase);

        result = executor.advance();
        assert!(result.is_err());
        assert_eq!(RUN_PHASE, executor.phase);
    }

    #[test]
    fn test_error_message() {
        let (plugin, _) = set_up();
        let executor = Executor::new(plugin);

        let msg = unsafe { executor.error_message(0) };
        assert_eq!("foo", msg.unwrap());
    }

    #[test]
    fn test_attribute_count() {
        let (plugin, _) = set_up();
        let executor = Executor::new(plugin);

        let count = if let Ok(count) = executor.attribute_count() {
            count
        } else {
            panic!("Could not obtain attribute count")
        };

        assert_eq!(1, count);
    }

    #[test]
    fn test_attribute_ids() {
        let (plugin, _) = set_up();
        let executor = Executor::new(plugin);

        let ids = if let Ok(ids) = executor.attribute_ids() {
            ids
        } else {
            panic!("Could not obtain attribute ids")
        };

        assert_eq!(1, ids.len());
        assert_eq!(0, ids[0]);
    }

    #[test]
    fn test_attribute_name() {
        let (mut plugin, _) = set_up();
        let cases: Vec<(Result<String, ExecutorError>, AttributeName)> = vec![
            (Ok(String::from("")), attribute_name_ok),
            (
                Err(NameError::DoesNotExist(String::from("foo")).into()),
                attribute_name_does_not_exist,
            ),
            (
                Err(NameError::Failure(String::from("foo")).into()),
                attribute_name_failure,
            ),
        ];

        let mut result: Result<String, ExecutorError>;
        let mut executor: Executor;
        for (expected, case) in cases {
            plugin.vtable.attribute_name = case;
            executor = Executor::new(plugin.clone());

            result = executor.attribute_name(0);
            match (expected, result) {
                (Ok(exp), Ok(res)) => assert_eq!(exp, res),
                (Err(exp), Err(res)) => assert_eq!(exp, res),
                _ => panic!("Result types differ"),
            }
        }

        tear_down(plugin);
    }

    #[test]
    fn test_attribute_value() {
        let (mut plugin, _) = set_up();
        let cases: Vec<(Result<(), ExecutorError>, AttributeValue)> = vec![
            (Ok(()), attribute_value_ok),
            (
                Err(ValueError::DoesNotExist(String::from("foo")).into()),
                attribute_value_does_not_exist,
            ),
            (
                Err(ValueError::Failure(String::from("foo")).into()),
                attribute_value_failure,
            ),
        ];

        let mut executor: Executor;
        let mut value = Val::Int(0);
        let mut result: Result<(), ExecutorError>;
        for (expected, case) in cases {
            plugin.vtable.attribute_value = case;
            executor = Executor::new(plugin.clone());

            result = executor.attribute_value(0, &mut value);
            assert_eq!(expected, result);
        }

        tear_down(plugin);
    }

    #[test]
    fn test_discover_attributes() {
        let (plugin, _) = set_up();
        let mut executor = Executor::new(plugin);
        let attribute = Attribute::new(Val::Int(42), 0, String::from("bar"), true);

        let attrs = executor.discover_attributes().unwrap();
        assert_eq!(&attribute.unwrap(), attrs.get(&0).unwrap());
    }

    fn set_up() -> (Plugin, Peripheral) {
        let plugin_data = Box::into_raw(Box::new(MockPluginData {})) as *mut PluginData;
        let vtable = VTable {
            plugin_free: def_peripheral_free,
            plugin_init: def_plugin_init,
            error_message_ns: def_error_message,
            attribute_count: def_attribute_count,
            attribute_ids: def_attribute_ids,
            attribute_name: def_attribute_name,
            attribute_pre_init: def_attribute_pre_init,
            attribute_value: def_attribute_value,
            set_attribute_value: def_set_attribute_value,
        };
        let plugin = Plugin {
            plugin_data,
            vtable,
        };

        let builder: PeripheralBuilder = PeripheralBuilder::new(0, "foo".to_string());
        let model = builder.set_id(0).build().unwrap();

        (plugin, model)
    }

    fn tear_down(plugin: Plugin) {
        unsafe { Box::from_raw(plugin.plugin_data) };
    }

    struct MockPluginData {}

    // Default function pointers for the vtable
    extern "C" fn def_peripheral_free(_: *mut PluginData) {}

    extern "C" fn def_plugin_init(_: *mut PluginData) -> c_int {
        0
    }

    extern "C" fn def_error_message(_: c_int) -> *const c_uchar {
        b"foo\0" as *const c_uchar
    }

    extern "C" fn def_attribute_count(_: *const PluginData, count: *mut size_t) -> c_int {
        unsafe { *count = 1 };
        PLUGIN_OK
    }

    extern "C" fn def_attribute_ids(
        _: *const PluginData,
        buffer: *mut size_t,
        _length: size_t,
    ) -> c_int {
        unsafe {
            let ids: &[usize] = &[0usize];
            let buffer = std::slice::from_raw_parts_mut(buffer, 1);
            buffer[0..1].copy_from_slice(ids);
        };
        PLUGIN_OK
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
    extern "C" fn def_attribute_pre_init(_: *const PluginData, _: size_t, _: *mut c_char) -> c_int {
        PLUGIN_OK
    }
    extern "C" fn def_attribute_value(
        _: *const PluginData,
        id: size_t,
        value: *mut Val,
        _: Phase,
    ) -> c_int {
        if id == 0 {
            unsafe { *value = Val::Int(42) };
            PLUGIN_OK
        } else {
            ATTRIBUTE_DOES_NOT_EXIST
        }
    }
    extern "C" fn def_set_attribute_value(
        _: *mut PluginData,
        _: size_t,
        _: *const Val,
        _: Phase,
    ) -> c_int {
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
    extern "C" fn attribute_value_ok(
        _: *const PluginData,
        _: size_t,
        _: *mut Val,
        _: Phase,
    ) -> c_int {
        PLUGIN_OK
    }
    extern "C" fn attribute_value_does_not_exist(
        _: *const PluginData,
        _: size_t,
        _: *mut Val,
        _: Phase,
    ) -> c_int {
        ATTRIBUTE_DOES_NOT_EXIST
    }
    extern "C" fn attribute_value_failure(
        _: *const PluginData,
        _: size_t,
        _: *mut Val,
        _: Phase,
    ) -> c_int {
        999
    }
}
