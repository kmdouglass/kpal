pub mod constants {
    use libc::c_int;

    pub const LIBRARY_OK: c_int = 0;
    pub const LIBRARY_ERR: c_int = 1;

    pub const PERIPHERAL_OK: c_int = 0;
    pub const PERIPHERAL_ERR: c_int = 1;
    pub const PERIPHERAL_ATTRIBUTE_DOES_NOT_EXIST: c_int = 2;
    pub const PERIPHERAL_COULD_NOT_SET_ATTRIBUTE: c_int = 3;
}
pub mod strings;

use std::cmp::{Eq, PartialEq};
use std::error;
use std::ffi::CString;
use std::fmt;

use libc::{c_double, c_int, c_long, c_uchar, size_t};

#[derive(Debug)]
#[repr(C)]
pub struct Peripheral {
    _private: [u8; 0],
}

#[derive(Debug)]
#[repr(C)]
pub struct VTable {
    pub peripheral_free: extern "C" fn(*mut Peripheral),
    pub attribute_name: extern "C" fn(
        peripheral: *const Peripheral,
        id: size_t,
        buffer: *mut c_uchar,
        length: size_t,
    ) -> c_int,
    pub attribute_value:
        extern "C" fn(peripheral: *const Peripheral, id: size_t, value: *mut Value) -> c_int,
    pub set_attribute_value:
        extern "C" fn(peripheral: *mut Peripheral, id: size_t, value: *const Value) -> c_int,
}

pub type PeripheralNew = extern "C" fn() -> *mut Peripheral;
pub type VTableNew = extern "C" fn() -> VTable;

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

#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
pub enum Value {
    Int(c_long),
    Float(c_double),
}

pub type Result<T> = std::result::Result<T, AttributeError>;

/// An AttributeError is raised when there is a failure to get or set a attribute's value.
#[derive(Debug)]
pub struct AttributeError {
    action: Action,
    error_code: c_int,
    message: String,
}

impl AttributeError {
    pub fn new(action: Action, error_code: c_int, message: &str) -> AttributeError {
        AttributeError {
            action: action,
            error_code: error_code,
            message: String::from(message),
        }
    }

    pub fn error_code(&self) -> c_int {
        self.error_code
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
