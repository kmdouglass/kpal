use std::boxed::Box;
use std::ffi::CString;
use std::ptr;

use libc::{c_char, c_void, size_t};

use kpal_peripheral::error::Error;
use kpal_peripheral::{Peripheral, Property, Value};

struct Basic {
    error: Error,
    props: Vec<Property>,
}

impl Peripheral for Basic {
    fn new() -> Basic {
        Basic {
            error: Error::new(),
            props: vec![
                Property {
                    name: "x",
                    value: Value::_Float(0.0),
                },
                Property {
                    name: "y",
                    value: Value::_Int(0),
                },
                Property {
                    name: "z",
                    value: Value::_String(
                        CString::new("0").expect("Error: CString::new()").into_raw(),
                    ),
                },
            ],
        }
    }

    fn error(&mut self) -> &mut Error {
        &mut self.error
    }

    fn property_name(&self, id: usize) -> &str {
        self.props[id].name
    }

    fn property_value(&self, id: usize) -> &Value {
        &self.props[id].value
    }

    fn property_set_value(&mut self, id: usize, value: Value) {
        use Value::*;
        match (self.props[id].value, value) {
            (_Int(_), _Int(_)) => self.props[id].value = value,
            (_Float(_), _Float(_)) => self.props[id].value = value,
            (_String(_), _String(_)) => self.props[id].value = value,
            _ => self.error.set(
                CString::new("value's variant does not match that of property.")
                    .expect("Error: CString::new()"),
            ),
        }
    }
}

/// The following defines the C-API for interfacing with the peripheral.
///
///    peripheral_new: extern "C" fn() -> *mut c_void,
///    peripheral_free: extern "C" fn(*mut c_void),
///    peripheral_error: extern "C" fn(*mut c_void) -> *const c_char,
///    property_name: extern "C" fn(*const c_void, size_t) -> *const c_char,
///    property_value: extern "C" fn(*const c_void, size_t) -> *const Value,
///    property_set_value: extern "C" fn(*mut c_void, size_t, *const Value),
///

// TODO Generate the C-bindings in a macro
#[no_mangle]
extern "C" fn peripheral_new() -> *mut c_void {
    let peripheral: Box<Box<dyn Peripheral>> = Box::new(Box::new(Basic::new()));
    Box::into_raw(peripheral) as *mut c_void
}

#[no_mangle]
extern "C" fn peripheral_free(peripheral: *mut c_void) {
    if peripheral.is_null() {
        return;
    }
    let peripheral = peripheral as *mut Box<dyn Peripheral>;
    unsafe {
        Box::from_raw(peripheral);
    }
}

#[no_mangle]
extern "C" fn peripheral_error(peripheral: *mut c_void) -> *const c_char {
    assert!(!peripheral.is_null());
    let peripheral = peripheral as *mut Box<dyn Peripheral>;
    let peripheral = unsafe { &mut (*peripheral) };

    let error = peripheral.error();
    let msg = error.query();
    match msg {
        Some(msg) => msg.as_ptr(),
        None => ptr::null(),
    }
}

#[no_mangle]
extern "C" fn property_name(peripheral: *const c_void, id: size_t) -> *const c_char {
    assert!(!peripheral.is_null());
    let peripheral = peripheral as *const Box<dyn Peripheral>;
    let peripheral = unsafe { &(*peripheral) };
    CString::new(peripheral.property_name(id))
        .expect("Error: CString::new()")
        .into_raw()
}

#[no_mangle]
extern "C" fn property_value(peripheral: *const c_void, id: size_t) -> *const Value {
    assert!(!peripheral.is_null());
    let peripheral = peripheral as *const Box<dyn Peripheral>;
    let peripheral = unsafe { &(*peripheral) };
    peripheral.property_value(id) as *const Value
}

#[no_mangle]
extern "C" fn property_set_value(peripheral: *mut c_void, id: size_t, value: *const Value) {
    if peripheral.is_null() || value.is_null() {
        return;
    }
    let peripheral = peripheral as *mut Box<dyn Peripheral>;
    let (peripheral, value) = unsafe { (&mut *peripheral, *value) };
    peripheral.property_set_value(id, value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_property_value() {
        let mut peripheral = Basic::new();
        let new_values = vec![
            Value::_Float(3.14),
            Value::_Int(4),
            Value::_String(
                CString::new("pi")
                    .expect("Error: CString::new()")
                    .into_raw(),
            ),
        ];

        // Test setting each property to the new value
        for (i, value) in new_values.into_iter().enumerate() {
            peripheral.property_set_value(i, value);
            assert_eq!(
                value, peripheral.props[i].value,
                "Expected property value to be {:?} but it was {:?}",
                value, peripheral.props[i].value
            )
        }
    }

    #[test]
    fn set_property_wrong_variant() {
        let mut peripheral = Basic::new();
        let new_value = Value::_Float(42.0);

        peripheral.property_set_value(1, new_value);
        let error = peripheral.error.query();
        match error {
            Some(_msg) => (),
            None => panic!("Expected a triggered error state."),
        }
    }
}
