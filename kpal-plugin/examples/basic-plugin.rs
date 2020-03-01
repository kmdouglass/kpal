//! A basic example of a plugin library with data but no hardware routines.
//!
//! This example demonstrates how to write a minimal plugin library that is capable of
//! communicating with a fake plugin consisting of only a Rust struct. It does not communicate with
//! any actual hardware. Its primary purpose is for demonstration and testing.
//!
//! A library such as this one consists of four parts:
//!
//! 1. a struct which contains the plugin's data
//! 2. a set of methods that the struct implements for manipulating the data and communicating with
//! the plugin
//! 3. initialization routines that are exposed through the C API
//! 4. a set of functions that comprise the plugin API
// Import any needed items from the standard and 3rd party libraries.
use std::{
    boxed::Box,
    cell::RefCell,
    convert::TryInto,
    error::Error,
    ffi::CString,
    fmt,
    time::{SystemTime, UNIX_EPOCH}, // These are used to generate a random number for an example.
};

use libc::c_int;

// Import the tools provided by the plugin library.
use kpal_plugin::{error_codes::*, *};

/// The first component of a plugin library is a struct that contains the plugin's data.
///
/// In this example, the struct contains only one field, `attributes`, which contains the list of
/// all attributes provided by the plugin. In general, it can contain any number of fields that are
/// necessary for the plugin.
#[derive(Debug)]
#[repr(C)]
struct Basic {
    /// A Vec of attributes that describe the peripheral's state.
    ///
    /// We wrap the attributes in a RefCell so that we can mutate their values inside methods where
    /// instances of this struct are immutable.
    attributes: Attributes<Self, BasicError>,
}

// Plugins implement the PluginAPI trait. They take a custom error type as a type parameter that is
// also provided by the library (see below).
impl PluginAPI<BasicError> for Basic {
    /// Returns a new instance of the plugin. No initialization of the hardware is performed.
    fn new() -> Result<Basic, BasicError> {
        Ok(Basic {
            attributes: RefCell::new(multimap! {
                0, "x" => Attribute {
                    name: CString::new("x").unwrap(),
                    value: Value::Double(0.0),

                    // Init callbacks are used during the initialization phase of a plugin to
                    // configure it before use.
                    callbacks_init: Callbacks::Update,

                    // Settable attributes should use the GetAndSet Callback variant.
                    callbacks_run: Callbacks::GetAndSet(on_get_x, on_set_x),
                },
                1, "y" => Attribute {
                    name: CString::new("y").unwrap(),
                    value: Value::Int(0),

                    // Constant init callbacks do not change value what-so-ever during the init
                    // phase of a plugin. Their value during this phase will be the same as the
                    // default value defined above.
                    callbacks_init: Callbacks::Constant,

                    // Not all attributes can be set. For example, the value of a sensor may only
                    // be readable. For these attributes, use the Get variant.
                    callbacks_run: Callbacks::Get(on_get_y),
                },
                2, "z" => Attribute {
                    name: CString::new("z").unwrap(),
                    value: Value::Int(42),
                    callbacks_init: Callbacks::Constant,
                    // Attributes that are constant during the run phase of a plugin should use the
                    // Constant variant of the Callbacks enum. They are not settable and will
                    // always return the same value.
                    callbacks_run: Callbacks::Constant,
                },
                3, "msg" => Attribute {
                    name: CString::new("msg").unwrap(),
                    // Values can contain CStrings as well. A CString is ASCII-encoded and ends in
                    // a null byte.
                    value: Value::String(CString::new("foobar").unwrap()),
                    callbacks_init: Callbacks::Constant,
                    callbacks_run: Callbacks::GetAndSet(on_get_msg, on_set_msg),
                },
            }),
        })
    }

    /// Initializes the plugin by performing any hardware initialzation.
    fn init(&mut self) -> Result<(), BasicError> {
        // Typically we'd actually setup the hardware here, but since this is an example that is
        // not attached to any real hardware, we just print a message instead.
        println!("Initializing the BasicPlugin... Done!");

        Ok(())
    }

    /// Returns the attributes of the plugin.
    ///
    /// This method must be defined by a plugin library because the PluginAPI trait cannot specify
    /// the name of the field of the Basic struct that stores the attributes.
    fn attributes(&self) -> &Attributes<Basic, BasicError> {
        &self.attributes
    }
}

// Callbacks are used to acutally communicate with the hardware whenever an attribute is read or
// set. Each settable attribute needs its own pair of callbacks, one for getting the value of the
// attribute and one for setting its value.
/// Callback function that is fired when the 'x' attribute is read during the run phase.
///
/// # Arguments
///
/// * `_plugin` - A reference to the plugin struct. This provides the callback with the plugin's
/// state.
/// * `cached` - The most recently read or modified value of the attribute.
fn on_get_x(_plugin: &Basic, _cached: &Value) -> Result<Value, BasicError> {
    // Normally, we would communicate with the hardware here to get the value of the
    // attribute. Since this is an example plugin, however, we just print a message to the terminal
    // instead and return the cached value of the attribute.
    //
    // In a real plugin, you could use the cached argument to avoid communicating with the hardware
    // if some condition is true. For example, you could store a boolean value in the plugin struct
    // and, only if it is true, return cached without querying the hardware.
    println!("Getting the value of attribute x");

    Ok(_cached.clone())
}

/// Callback function that is fired when the 'x' attribute is set during the run phase.
///
/// # Arguments
///
/// * `_plugin` - A reference to the plugin struct. This provides the callback with the plugin's
/// state.
/// * `cached` - The most most recently read or modified value of the attribute.
/// * `val` - The new value of the attribute.
fn on_set_x(_plugin: &Basic, _cached: &Value, _val: &Val) -> Result<(), BasicError> {
    // Like in the callback above, in this example plugin we only print a line to the console. The
    // update of the cached value will be taken care of for you.
    println!("Setting the value of attribute x");

    Ok(())
}

/// Callback function that is fired when the 'y' attribute is read.
///
/// Not every attribute needs a callback for setting its value. Here, we demonstrate this by
/// defining only a `get` callback for the attribute `y`.
///
/// # Arguments
///
/// * `_plugin` - A reference to the plugin struct. This provides the callback with the plugin's
/// state.
/// * `cached` - The most most recently read or modified value of the attribute.
fn on_get_y(_plugin: &Basic, _cached: &Value) -> Result<Value, BasicError> {
    println!("Getting the value of attribute y");
    // This simulates a random value from a sensor; its implementation does not matter for the
    // purpose of this example.
    let rand_int: c_int = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
        .try_into()
        .unwrap_or(42);

    let value = Value::Int(rand_int);

    Ok(value)
}

/// Getting String attributes works just like those with numeric values.
///
/// # Arguments
///
/// * `_plugin` - A reference to the plugin struct. This provides the callback with the plugin's
/// state.
/// * `cached` - The most most recently read or modified value of the attribute.
fn on_get_msg(_plugin: &Basic, _cached: &Value) -> Result<Value, BasicError> {
    println!("Getting the value of attribute msg");

    Ok(_cached.clone())
}

/// Setting String attributes works just like those with numeric values.
///
/// # Arguments
///
/// * `_plugin` - A reference to the plugin struct. This provides the callback with the plugin's
/// state.
/// * `cached` - The most most recently read or modified value of the attribute.
/// * `val` - The new value of the attribute.
fn on_set_msg(_plugin: &Basic, _cached: &Value, _val: &Val) -> Result<(), BasicError> {
    println!("Setting the value of attribute msg");

    Ok(())
}

/// The plugin's error type.
///
/// Plugin methods all return the same, custom error type provided by the plugin author(s). This
/// allows developers to pack any information that they wish into the error type. In addition, by
/// providing their own error, plugin authors can implement the From trait to automatically
/// transform errors raised in the PluginAPI methods into this type with the `?` operator.
#[derive(Debug)]
struct BasicError {
    error_code: c_int,
}

impl Error for BasicError {}

impl fmt::Display for BasicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Basic error {{ error_code: {} }}", self.error_code)
    }
}

/// KPAL requires that the library's custom error implement the PluginError trait
///
/// This ensures that required information is always passed back to the daemon.
impl PluginError for BasicError {
    /// Intializes and returns a new instance of the plugin's error type.
    fn new(error_code: c_int) -> BasicError {
        BasicError { error_code }
    }

    /// Returns the error code associated with the plugin's error type.
    ///
    /// This code should be one of the values found in the `constants` module.
    fn error_code(&self) -> c_int {
        self.error_code
    }
}

// Now that everything has been setup, we call the `declare_plugin` macro to automatically generate
// the functions and structures that will be used by the daemon to communicate with the
// plugin. This macro allows plugin developers to entirely avoid writing unsafe, foreign function
// interface code.
declare_plugin!(Basic, BasicError);

// Unit tests for the plugin lie within a module called `tests` that is preceded by a #[cfg(test)]
// attribute.
#[cfg(test)]
mod tests {
    use libc::c_uchar;

    use crate::RUN_PHASE;

    use super::*;

    #[test]
    fn test_kpal_error() {
        struct Case {
            description: &'static str,
            error_code: c_int,
            want_null: bool,
        };

        let cases = vec![
            Case {
                description: "a valid error code is passed to kpal_error",
                error_code: 0,
                want_null: false,
            },
            Case {
                description: "an invalid and negative error code is passed to kpal_error",
                error_code: -1,
                want_null: true,
            },
            Case {
                description: "an invalid and positive error code is passed to kpal_error",
                error_code: 99999,
                want_null: true,
            },
        ];

        let mut msg: *const c_uchar;
        for case in &cases {
            log::info!("{}", case.description);
            msg = error_message_ns(case.error_code);

            if case.want_null {
                assert!(msg.is_null());
            } else {
                assert!(!msg.is_null());
            }
        }
    }

    #[test]
    fn set_attribute_value() {
        let plugin = Basic::new().unwrap();
        let new_val = Val::Double(3.14);

        // Test setting each attribute to the new value
        plugin.attribute_set_value(0, &new_val, RUN_PHASE).unwrap();
        let attributes = plugin.attributes.borrow();
        let actual = &attributes.get(&0).unwrap().value.as_val();
        assert_eq!(
            new_val, *actual,
            "Expected attribute value to be {:?} but it was {:?}",
            new_val, *actual
        )
    }

    #[test]
    fn set_attribute_wrong_variant() {
        let plugin = Basic::new().unwrap();
        let new_val = Val::Double(42.0);

        let result = plugin.attribute_set_value(1, &new_val, RUN_PHASE);
        match result {
            Ok(_) => panic!("Expected different value variants."),
            Err(_) => (),
        }
    }
}
