//! The KPAL plugin crate provides tools to write your own KPAL plugins.
//!
//! See the examples folder for ideas on how to implement the datatypes and methods defined in this
//! library.
mod errors;
mod ffi;
mod strings;

use std::cmp::{Eq, PartialEq};
use std::error::Error;
use std::ffi::{CStr, CString};

use libc::{c_double, c_int, c_long, c_uchar, size_t};

pub use errors::constants;
pub use errors::ERRORS;
pub use ffi::*;
pub use strings::copy_string;

/// The set of functions that must be implemented by a plugin.
pub trait PluginAPI<E: Error + PluginError> {
    type Plugin;

    /// Initializes and returns a new instance of the plugin.
    fn new() -> Result<Self::Plugin, E>;

    /// Returns the name of an attribute of the plugin.
    fn attribute_name(&self, id: usize) -> Result<&CStr, E>;

    /// Returns the value of an attribute of the plugin.
    fn attribute_value(&self, id: usize) -> Result<Value, E>;

    /// Sets the value of an attribute.
    fn attribute_set_value(&mut self, id: usize, value: &Value) -> Result<(), E>;
}

/// The set of functions that must be implemented by a plugin library's main error type.
pub trait PluginError: std::error::Error {
    /// Returns the error code of the instance.
    fn error_code(&self) -> c_int;
}

/// A Plugin combines the data that represents a peripheral's state and with its functionality.
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
    /// A pointer to a struct containing the state of a peripheral.
    pub peripheral: *mut Peripheral,

    /// The table of function pointers that define part of the plugin API.
    pub vtable: VTable,
}

impl Drop for Plugin {
    /// Frees the memory allocated to the peripheral data.
    fn drop(&mut self) {
        (self.vtable.peripheral_free)(self.peripheral);
    }
}

unsafe impl Send for Plugin {}

/// An opaque struct that contains the state of an individual peripheral.
///
/// In Rust, an opaque struct is defined as a struct with a field that is a zero-length array of
/// unsigned 8-bit integers. It is used to hide the peripheral's state, forcing all interactions
/// with the data through the functions in the vtable instead.
#[derive(Debug)]
#[repr(C)]
pub struct Peripheral {
    _private: [u8; 0],
}

/// A table of function pointers that comprise the plugin API for the foreign function interface.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct VTable {
    /// Frees the memory associated with a peripheral.
    pub peripheral_free: extern "C" fn(*mut Peripheral),

    /// Returns an error message associated with a Plugin error code.
    pub error_message: extern "C" fn(c_int) -> *const c_uchar,

    /// Writes the name of an attribute to a buffer that is provided by the caller.
    pub attribute_name: unsafe extern "C" fn(
        peripheral: *const Peripheral,
        id: size_t,
        buffer: *mut c_uchar,
        length: size_t,
    ) -> c_int,

    /// Writes the value of an attribute to a Value instance that is provided by the caller.
    pub attribute_value:
        unsafe extern "C" fn(peripheral: *const Peripheral, id: size_t, value: *mut Value) -> c_int,

    /// Sets the value of an attribute.
    pub set_attribute_value:
        unsafe extern "C" fn(peripheral: *mut Peripheral, id: size_t, value: *const Value) -> c_int,
}

/// The type signature of the function that returns a new plugin instance.
pub type KpalPluginInit = unsafe extern "C" fn(*mut Plugin) -> c_int;

/// The type signature of the function that initializes a library.
pub type KpalLibraryInit = unsafe extern "C" fn() -> c_int;

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
