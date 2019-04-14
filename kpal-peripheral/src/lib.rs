// TODO Create a macro to generate properties inside a device library
pub mod error;

use std::cmp::{Eq, PartialEq};

use libc::{c_char, c_double, int32_t};

pub trait KpalApiV0<T> {
    fn kpal_api_new() -> *mut T;
    fn kpal_api_free(kpal_api: *mut T);

    fn kpal_property_name(&self, id: usize) -> &str;
    fn kpal_property_value(&self, id: usize) -> &Value;
    fn kpal_property_set_value(&mut self, id: usize, value: Value);
}

/// A property is a value that may be read from or set on a peripheral.
///
/// The set of all property values of a peripheral represent all that is known to the user about
/// the peripheral's state.
#[derive(Debug)]
#[repr(C)]
pub struct Property {
    pub name: &'static str,
    pub value: Value,
}

impl Eq for Property {}

impl PartialEq for Property {
    fn eq(&self, other: &Property) -> bool {
        self.name == other.name
    }
}

/// A value represents the current state of a property.
///
/// Values are passed through the C ABI. As a result, the variants of a Value must hold
/// C-compatible data types.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub enum Value {
    _Int(int32_t),
    _Float(c_double),
    _String(*const c_char),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn properties_with_the_same_name_and_same_values_are_equal() {
        let prop1_left = Property {
            name: "prop1",
            value: Value::_Int(0),
        };
        let prop1_right = Property {
            name: "prop1",
            value: Value::_Int(0),
        };

        assert_eq!(prop1_left, prop1_right);
    }

    #[test]
    fn properties_with_the_same_name_and_different_values_are_equal() {
        let prop1_left = Property {
            name: "prop1",
            value: Value::_Int(0),
        };
        let prop1_right = Property {
            name: "prop1",
            value: Value::_Int(1),
        };

        assert_eq!(prop1_left, prop1_right);
    }

    #[test]
    fn properties_with_different_names_are_not_equal() {
        let prop1 = Property {
            name: "prop1",
            value: Value::_Int(0),
        };
        let prop2 = Property {
            name: "prop2",
            value: Value::_Int(0),
        };

        assert_ne!(prop1, prop2);
    }
}
