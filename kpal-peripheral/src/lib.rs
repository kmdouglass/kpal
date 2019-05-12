pub mod constants {
    use libc::c_int;

    pub const PERIPHERAL_OK: c_int = 0;
    pub const PERIPHERAL_ERR: c_int = -1;
}

// TODO Create a macro to generate properties inside a device library
use std::cmp::{Eq, PartialEq};
use std::error;
use std::ffi::{CStr, CString};
use std::fmt;
use std::sync::{Arc, Mutex};

pub trait Peripheral: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    fn property_name(&self, id: usize) -> Result<&CStr>;
    fn property_value(&self, id: usize) -> Result<Value>;
    fn property_set_value(&self, id: usize, value: &Value) -> Result<()>;
}

/// A property is a value that may be read from or set on a peripheral.
///
/// The set of all property values of a peripheral represent all that is known to the user about
/// the peripheral's state.
#[derive(Debug)]
#[repr(C)]
pub struct Property {
    pub name: CString,
    pub value: Arc<Mutex<Value>>,
}

impl Eq for Property {}

impl PartialEq for Property {
    fn eq(&self, other: &Property) -> bool {
        self.name == other.name
    }
}

/// A value represents the current state of a property.
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
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
            value: Arc::new(Mutex::new(Value::Integer(0))),
        };
        let prop1_right = Property {
            name: CString::new("prop1").unwrap(),
            value: Arc::new(Mutex::new(Value::Integer(0))),
        };

        assert_eq!(prop1_left, prop1_right);
    }

    #[test]
    fn properties_with_the_same_name_and_different_values_are_equal() {
        let prop1_left = Property {
            name: CString::new("prop1").unwrap(),
            value: Arc::new(Mutex::new(Value::Integer(0))),
        };
        let prop1_right = Property {
            name: CString::new("prop1").unwrap(),
            value: Arc::new(Mutex::new(Value::Integer(1))),
        };

        assert_eq!(prop1_left, prop1_right);
    }

    #[test]
    fn properties_with_different_names_are_not_equal() {
        let prop1 = Property {
            name: CString::new("prop1").unwrap(),
            value: Arc::new(Mutex::new(Value::Integer(0))),
        };
        let prop2 = Property {
            name: CString::new("prop2").unwrap(),
            value: Arc::new(Mutex::new(Value::Integer(0))),
        };

        assert_ne!(prop1, prop2);
    }
}
