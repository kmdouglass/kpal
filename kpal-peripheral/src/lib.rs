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
pub struct Property<'a> {
    pub name: &'a str,
    pub value: Value,
    //callback: fn(),  // TODO Make this a closure
}

impl<'a> Eq for Property<'a> {}

impl<'a> PartialEq for Property<'a> {
    // TODO(?) Write a test for this
    fn eq(&self, other: &Property) -> bool {
        self.name == other.name
    }
}

/// A value represents the current state of a property.
pub enum Value {
    Int(i32),
    Float(f64),
    String(String),
}
