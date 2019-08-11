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
    pub attribute_name: extern "C" fn(peripheral: *const Peripheral, id: size_t) -> *const c_char,
    pub attribute_value:
        extern "C" fn(peripheral: *const Peripheral, id: size_t, value: *mut Value) -> c_int,
    pub set_attribute_value:
        extern "C" fn(peripheral: *mut Peripheral, id: size_t, value: *const Value) -> c_int,
}

#[derive(Debug)]
#[repr(C)]
pub struct Attribute {
    pub name: CString,
    pub value: Value,
}

impl Eq for Attribute {}

impl PartialEq for Attribute {
    fn eq(&self, other: &Attribute) -> bool {
        self.name == other.name
    }
}

// TODO Change inner datatypes to be C-compatible
#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
pub enum Value {
    Int(i64),
    Float(f64),
}

pub type Result<T> = std::result::Result<T, AttributeError>;

/// An AttributeError is raised when there is a failure to get or set a attribute's value.
#[derive(Debug)]
pub struct AttributeError {
    action: Action,
    message: String,
}

impl AttributeError {
    pub fn new(action: Action, message: &str) -> AttributeError {
        AttributeError {
            action: action,
            message: String::from(message),
        }
    }
}

impl fmt::Display for AttributeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AttributeError {{ action: {:?}, message {} }}",
            self.action, self.message
        )
    }
}

impl error::Error for AttributeError {
    fn description(&self) -> &str {
        "failed to access attribute value"
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
