//! The KPAL plugin crate provides tools to write your own KPAL plugins.
//!
//! See the examples folder for ideas on how to implement the datatypes and methods defined in this
//! library.
pub mod constants {
    // TODO Move these inside the errors module and get rid of the constants module
    //! Constants and return codes used by Plugins to communicate the result of function calls.
    use libc::c_int;

    pub const PLUGIN_OK: c_int = 0;
    pub const UNDEFINED_ERR: c_int = 1;
    pub const PLUGIN_INIT_ERR: c_int = 2;
    pub const ATTRIBUTE_DOES_NOT_EXIST: c_int = 3;
    pub const ATTRIBUTE_TYPE_MISMATCH: c_int = 4;
    pub const IO_ERR: c_int = 5;
    pub const NUMERIC_CONVERSION_ERR: c_int = 6;
}
mod errors;
mod ffi;
mod strings;

use std::cmp::{Eq, PartialEq};
use std::error::Error;
use std::ffi::{CStr, CString};

use libc::{c_double, c_int, c_long, c_uchar, size_t};

pub use errors::ERRORS;
pub use ffi::*;
pub use strings::copy_string;

// TODO Add doc strings
pub trait PluginAPI<E: Error + PluginError> {
    type Plugin;

    fn new() -> Result<Self::Plugin, E>;
    fn attribute_name(&self, id: usize) -> Result<&CStr, E>;
    fn attribute_value(&self, id: usize) -> Result<Value, E>;
    fn attribute_set_value(&mut self, id: usize, value: &Value) -> Result<(), E>;
}

pub trait PluginError: std::error::Error {
    /// Returns the error code of the instance.
    fn error_code(&self) -> c_int;
}

/// A Plugin contains the necessary data to work with a plugin across the FFI boundary.
///
/// This struct holds a raw pointer to a peripheral that is created by the plugin library. In
/// addition, it contains the vtable of function pointers defined by the C API and implemented
/// within the plugin library.
///
/// # Safety
///
/// The plugin implements the `Send` trait because after creation the plugin is moved into the
/// thread that is dedicated to the peripheral that it manages. Once it is moved, it will only ever
/// be owned and used by this single thread by design.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Plugin {
    // TODO Rename this to avoid confusion with the user API's Peripheral (and peripheral_free)
    // Use plugin_data
    /// A pointer to a peripheral instance.
    pub peripheral: *mut Peripheral,

    /// The table of function pointers that define part of the plugin API.
    pub vtable: VTable,
}

impl Drop for Plugin {
    /// Frees the memory allocated to the plugin.
    fn drop(&mut self) {
        (self.vtable.peripheral_free)(self.peripheral);
    }
}

unsafe impl Send for Plugin {}

/// An opaque struct that contains the peripheral's data.
///
/// In Rust, an opaque struct is defined as a struct with a field that is a zero-length array of
/// unsigned 8-bit integers. It is used to hide the peripheral's data, forcing all interactions
/// with the data through the functions in the vtable instead.
#[derive(Debug)]
#[repr(C)]
pub struct Peripheral {
    _private: [u8; 0],
}

/// A table of function pointers that comprise the plugin API.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct VTable {
    /// Frees the memory associated with a peripheral.
    pub peripheral_free: extern "C" fn(*mut Peripheral),

    /// Returns an error message associated with a Plugin error code.
    pub error_message: extern "C" fn(c_int) -> *const c_uchar,

    /// Writes the name of an attribute to a buffer that is provided by the caller.
    pub attribute_name: extern "C" fn(
        peripheral: *const Peripheral,
        id: size_t,
        buffer: *mut c_uchar,
        length: size_t,
    ) -> c_int,

    /// Writes the value of an attribute to a Value instance that is provided by the caller.
    pub attribute_value:
        extern "C" fn(peripheral: *const Peripheral, id: size_t, value: *mut Value) -> c_int,

    /// Sets the value of an attribute.
    pub set_attribute_value:
        extern "C" fn(peripheral: *mut Peripheral, id: size_t, value: *const Value) -> c_int,
}

/// The type signature of the function that returns a new plugin instance.
pub type KpalPluginInit = extern "C" fn(*mut Plugin) -> c_int;

/// A single piece of information that partly determines the state of a peripheral.
#[derive(Debug)]
#[repr(C)]
pub struct Attribute {
    /// The name of the attribute.
    pub name: CString,

    /// The value of the attribute.
    pub value: Value,
}

impl Eq for Attribute {}

impl PartialEq for Attribute {
    fn eq(&self, other: &Attribute) -> bool {
        self.name == other.name
    }
}

/// The value of an attribute.
///
/// Currently only integer and floating point (decimal) values are supported.
#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
pub enum Value {
    Int(c_long),
    Float(c_double),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn properties_with_the_same_name_and_same_values_are_equal() {
        let prop1_left = Attribute {
            name: CString::new("prop1").unwrap(),
            value: Value::Int(0),
        };
        let prop1_right = Attribute {
            name: CString::new("prop1").unwrap(),
            value: Value::Int(0),
        };

        assert_eq!(prop1_left, prop1_right);
    }

    #[test]
    fn properties_with_the_same_name_and_different_values_are_equal() {
        let prop1_left = Attribute {
            name: CString::new("prop1").unwrap(),
            value: Value::Int(0),
        };
        let prop1_right = Attribute {
            name: CString::new("prop1").unwrap(),
            value: Value::Int(1),
        };

        assert_eq!(prop1_left, prop1_right);
    }

    #[test]
    fn properties_with_different_names_are_not_equal() {
        let prop1 = Attribute {
            name: CString::new("prop1").unwrap(),
            value: Value::Int(0),
        };
        let prop2 = Attribute {
            name: CString::new("prop2").unwrap(),
            value: Value::Int(0),
        };

        assert_ne!(prop1, prop2);
    }
}
