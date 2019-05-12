use std::boxed::Box;
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::{Arc, Mutex};

use libc::{c_char, c_int, c_void, size_t};

use kpal_peripheral::constants::{PERIPHERAL_ERR, PERIPHERAL_OK};
use kpal_peripheral::{Action, Peripheral, Property, PropertyError, Result, Value};

struct Basic {
    props: Vec<Property>,
}

impl Peripheral for Basic {
    fn new() -> Basic {
        Basic {
            props: vec![
                Property {
                    name: CString::new("x").expect("Error creating CString"),
                    value: Arc::new(Mutex::new(Value::Float(0.0))),
                },
                Property {
                    name: CString::new("y").expect("Error creating CString"),
                    value: Arc::new(Mutex::new(Value::Integer(0))),
                },
                Property {
                    name: CString::new("z").expect("Error creating CString"),
                    value: Arc::new(Mutex::new(Value::String(String::from("0")))),
                },
            ],
        }
    }

    fn property_name(&self, id: usize) -> Result<&CStr> {
        match self.props.get(id) {
            Some(property) => Ok(&property.name),
            None => Err(PropertyError::new(
                Action::Get,
                &format!("Property index {} does not exist.", id),
            )),
        }
    }

    fn property_value(&self, id: usize) -> Result<Value> {
        let property = self.props.get(id).ok_or_else(|| {
            PropertyError::new(
                Action::Get,
                &format!("Property index {} does not exist.", id),
            )
        })?;

        let value = property.value.lock().map_err(|_| {
            PropertyError::new(
                Action::Get,
                &format!("Could not access lock for property {}", id),
            )
        })?;

        Ok((*value).clone())
    }

    /// Sets the value of the given property.
    fn property_set_value(&self, id: usize, value: &Value) -> Result<()> {
        use Value::*;

        let property = self.props.get(id).ok_or_else(|| {
            PropertyError::new(
                Action::Get,
                &format!("Property index {} does not exist.", id),
            )
        })?;

        let mut current_value = property.value.lock().map_err(|_| {
            PropertyError::new(
                Action::Get,
                &format!("Could not access lock for property {}", id),
            )
        })?;

        match (&*current_value, &value) {
            (Integer(_), Integer(_)) | (Float(_), Float(_)) | (String(_), String(_)) => {
                *current_value = (*value).clone();
                Ok(())
            }
            _ => Err(PropertyError::new(
                Action::Set,
                &format!("Could not set property {}", id),
            )),
        }
    }
}

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
extern "C" fn property_name(peripheral: *const c_void, id: size_t) -> *const c_char {
    assert!(!peripheral.is_null());
    let peripheral = peripheral as *const Box<dyn Peripheral>;
    unsafe {
        match (*peripheral).property_name(id) {
            Ok(name) => name.as_ptr(),
            Err(_) => return ptr::null(),
        }
    }
}

#[no_mangle]
extern "C" fn property_value(peripheral: *const c_void, id: size_t, value: *mut c_void) -> c_int {
    assert!(!peripheral.is_null());
    let peripheral = peripheral as *const Box<dyn Peripheral>;
    let value = value as *mut Value;
    unsafe {
        match (*peripheral).property_value(id) {
            Ok(new_value) => *value = new_value,
            Err(_) => return PERIPHERAL_ERR,
        };
    }

    PERIPHERAL_OK
}

#[no_mangle]
extern "C" fn property_set_value(
    peripheral: *const c_void,
    id: size_t,
    value: *const c_void,
) -> c_int {
    if peripheral.is_null() || value.is_null() {
        return PERIPHERAL_ERR;
    }
    let peripheral = peripheral as *const Box<dyn Peripheral>;
    let value = value as *const Value;

    unsafe {
        match (*peripheral).property_set_value(id, &*value) {
            Ok(_) => PERIPHERAL_OK,
            Err(_) => PERIPHERAL_ERR,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_property_value() {
        let peripheral = Basic::new();
        let new_values = vec![
            Value::Float(3.14),
            Value::Integer(4),
            Value::String(String::from("pi")),
        ];

        // Test setting each property to the new value
        for (i, value) in new_values.into_iter().enumerate() {
            peripheral.property_set_value(i, &value).unwrap();
            let actual = peripheral.props[i].value.lock().unwrap();
            assert_eq!(
                value, *actual,
                "Expected property value to be {:?} but it was {:?}",
                value, *actual
            )
        }
    }

    #[test]
    fn set_property_wrong_variant() {
        let peripheral = Basic::new();
        let new_value = Value::Float(42.0);

        let result = peripheral.property_set_value(1, &new_value);
        match result {
            Ok(_) => panic!("Expected a triggered error state."),
            Err(_) => (),
        }
    }
}
