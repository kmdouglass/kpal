// TODO Create a macro to generate properties inside a device library
// TODO Serialize properties to JSON

use std::cmp::{Eq,PartialEq};

pub trait Peripheral {
    /// Returns all the properties of the peripheral.
    fn properties(&self) -> Vec<Property>;
}

/// A property is a value that may be read from or set on a peripheral.
///
/// The set of all property values of a peripheral represent all that is known to the user about
/// the peripheral's state.
#[derive(Debug)]
pub struct Property<'a> {
    pub name: &'a str,
    pub value: Value,
    //callback: fn(),  // TODO Make this a closure
}

impl<'a> Eq for Property<'a> {}

impl<'a> PartialEq for Property<'a> {
    fn eq(&self, other: &Property) -> bool {
        self.name == other.name
    }
}

/// A value represents the current state of a property.
#[derive(Debug)]
pub enum Value {
    Int(i32),
    Float(f64),
    String(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn properties_with_the_same_name_and_same_values_are_equal() {
        let prop1_left = Property {
            name: "prop1",
            value: Value::Int(0),
        };
        let prop1_right = Property {
            name: "prop1",
            value: Value::Int(0),
        };

        assert_eq!(prop1_left, prop1_right);
    }

    #[test]
        fn properties_with_the_same_name_and_different_values_are_equal() {
        let prop1_left = Property {
            name: "prop1",
            value: Value::Int(0),
        };
        let prop1_right = Property {
            name: "prop1",
            value: Value::Int(1),
        };

        assert_eq!(prop1_left, prop1_right);
    }

    #[test]
        fn properties_with_different_names_are_not_equal() {
        let prop1 = Property {
            name: "prop1",
            value: Value::Int(0),
        };
        let prop2 = Property {
            name: "prop2",
            value: Value::Int(0),
        };

        assert_ne!(prop1, prop2);
    }
}
