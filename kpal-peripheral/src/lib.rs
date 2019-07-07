pub mod constants {
    use libc::c_int;

    pub const PERIPHERAL_OK: c_int = 0;
    pub const PERIPHERAL_ERR: c_int = -1;
}

use std::cmp::{Eq, PartialEq};
use std::error;
use std::ffi::CString;
use std::fmt;

use libc::{c_char, c_int, size_t};

#[repr(C)]
pub struct Peripheral {
    _private: [u8; 0],
}

#[repr(C)]
pub struct VTable {
    pub peripheral_new: extern "C" fn() -> *mut Peripheral,
    pub peripheral_free: extern "C" fn(*mut Peripheral),
    pub property_name: extern "C" fn(peripheral: *const Peripheral, id: size_t) -> *const c_char,
    pub property_value:
        extern "C" fn(peripheral: *const Peripheral, id: size_t, value: *mut Value) -> c_int,
    pub set_property_value:
        extern "C" fn(peripheral: *mut Peripheral, id: size_t, value: *const Value) -> c_int,
}

#[derive(Debug)]
#[repr(C)]
pub struct Property {
    pub name: CString,
    pub value: Value,
}

impl Eq for Property {}

impl PartialEq for Property {
    fn eq(&self, other: &Property) -> bool {
        self.name == other.name
    }
}

#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
pub enum Value {
    Int(i64),
    Float(f64),
}

pub type Result<T> = std::result::Result<T, PropertyError>;

/// A property error is raised when there is a failure to get or set a property's value.
#[derive(Debug)]
pub struct PropertyError {
    action: Action,
    message: String,
}

impl PropertyError {
    pub fn new(action: Action, message: &str) -> PropertyError {
        PropertyError {
            action: action,
            message: String::from(message),
        }
    }
}

impl fmt::Display for PropertyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PropertyError {{ action: {:?}, message {} }}",
            self.action, self.message
        )
    }
}

impl error::Error for PropertyError {
    fn description(&self) -> &str {
        "failed to access property value"
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Get,
    Set,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn properties_with_the_same_name_and_same_values_are_equal() {
        let prop1_left = Property {
            name: CString::new("prop1").unwrap(),
            value: Value::Int(0),
        };
        let prop1_right = Property {
            name: CString::new("prop1").unwrap(),
            value: Value::Int(0),
        };

        assert_eq!(prop1_left, prop1_right);
    }

    #[test]
    fn properties_with_the_same_name_and_different_values_are_equal() {
        let prop1_left = Property {
            name: CString::new("prop1").unwrap(),
            value: Value::Int(0),
        };
        let prop1_right = Property {
            name: CString::new("prop1").unwrap(),
            value: Value::Int(1),
        };

        assert_eq!(prop1_left, prop1_right);
    }

    #[test]
    fn properties_with_different_names_are_not_equal() {
        let prop1 = Property {
            name: CString::new("prop1").unwrap(),
            value: Value::Int(0),
        };
        let prop2 = Property {
            name: CString::new("prop2").unwrap(),
            value: Value::Int(0),
        };

        assert_ne!(prop1, prop2);
    }
}
