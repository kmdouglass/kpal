use std::boxed::Box;
use std::ffi::{CStr, CString};

use libc::{c_int, c_uchar, size_t};

use kpal_peripheral::constants::{PERIPHERAL_ERR, PERIPHERAL_OK};
use kpal_peripheral::utils::copy_string;
use kpal_peripheral::{Action, Attribute, AttributeError, Peripheral, Result, VTable, Value};

#[repr(C)]
struct Basic {
    props: Vec<Attribute>,
}

impl Basic {
    fn new() -> Basic {
        Basic {
            props: vec![
                Attribute {
                    name: CString::new("x").expect("Error creating CString"),
                    value: Value::Float(0.0),
                },
                Attribute {
                    name: CString::new("y").expect("Error creating CString"),
                    value: Value::Int(0),
                },
            ],
        }
    }

    fn attribute_name(&self, id: usize) -> Result<&CStr> {
        match self.props.get(id) {
            Some(attribute) => Ok(&attribute.name),
            None => Err(AttributeError::new(
                Action::Get,
                &format!("Attribute at index {} does not exist.", id),
            )),
        }
    }

    fn attribute_value(&self, id: usize) -> Result<Value> {
        let attribute = self.props.get(id).ok_or_else(|| {
            AttributeError::new(
                Action::Get,
                &format!("Attribute at index {} does not exist.", id),
            )
        })?;

        Ok(attribute.value.clone())
    }

    /// Sets the value of the attribute given by the id.
    fn attribute_set_value(&mut self, id: usize, value: &Value) -> Result<()> {
        use Value::*;

        let current_value = &mut self
            .props
            .get_mut(id)
            .ok_or_else(|| {
                AttributeError::new(
                    Action::Get,
                    &format!("Attribute at index {} does not exist.", id),
                )
            })?
            .value;

        match (&current_value, &value) {
            (Int(_), Int(_)) | (Float(_), Float(_)) => {
                *current_value = (*value).clone();
                Ok(())
            }
            _ => Err(AttributeError::new(
                Action::Set,
                &format!("Could not set attribute {}", id),
            )),
        }
    }
}

#[no_mangle]
pub extern "C" fn vtable() -> VTable {
    VTable {
        peripheral_free: peripheral_free,
        attribute_name: attribute_name,
        attribute_value: attribute_value,
        set_attribute_value: set_attribute_value,
    }
}

pub extern "C" fn peripheral_new() -> *mut Peripheral {
    let peripheral: Box<Basic> = Box::new(Basic::new());
    Box::into_raw(peripheral) as *mut Peripheral
}

extern "C" fn peripheral_free(peripheral: *mut Peripheral) {
    if peripheral.is_null() {
        return;
    }
    let peripheral = peripheral as *mut Box<Peripheral>;
    unsafe {
        Box::from_raw(peripheral);
    }
}

extern "C" fn attribute_name(
    peripheral: *const Peripheral,
    id: size_t,
    buffer: *mut c_uchar,
    length: size_t,
) -> c_int {
    assert!(!peripheral.is_null());
    let peripheral = peripheral as *const Box<Basic>;
    unsafe {
        let name: &[u8] = match (*peripheral).attribute_name(id) {
            Ok(name) => name.to_bytes_with_nul(),
            Err(_) => return PERIPHERAL_ERR,
        };

        match copy_string(name, buffer, length) {
            Ok(_) => PERIPHERAL_OK,
            Err(_) => PERIPHERAL_ERR,
        }
    }
}

extern "C" fn attribute_value(
    peripheral: *const Peripheral,
    id: size_t,
    value: *mut Value,
) -> c_int {
    assert!(!peripheral.is_null());
    let peripheral = peripheral as *const Box<Basic>;
    let value = value as *mut Value;
    unsafe {
        match (*peripheral).attribute_value(id) {
            Ok(new_value) => *value = new_value,
            Err(_) => return PERIPHERAL_ERR,
        };
    }

    PERIPHERAL_OK
}

extern "C" fn set_attribute_value(
    peripheral: *mut Peripheral,
    id: size_t,
    value: *const Value,
) -> c_int {
    if peripheral.is_null() || value.is_null() {
        return PERIPHERAL_ERR;
    }
    let peripheral = peripheral as *mut Box<Basic>;
    let value = value as *const Value;

    unsafe {
        match (*peripheral).attribute_set_value(id, &*value) {
            Ok(_) => PERIPHERAL_OK,
            Err(_) => PERIPHERAL_ERR,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_attribute_value() {
        let mut peripheral = Basic::new();
        let new_values = vec![Value::Float(3.14), Value::Int(4)];

        // Test setting each attribute to the new value
        for (i, value) in new_values.into_iter().enumerate() {
            peripheral.attribute_set_value(i, &value).unwrap();
            let actual = &peripheral.props[i].value;
            assert_eq!(
                value, *actual,
                "Expected attribute value to be {:?} but it was {:?}",
                value, *actual
            )
        }
    }

    #[test]
    fn set_attribute_wrong_variant() {
        let mut peripheral = Basic::new();
        let new_value = Value::Float(42.0);

        let result = peripheral.attribute_set_value(1, &new_value);
        match result {
            Ok(_) => panic!("Expected different value variants."),
            Err(_) => (),
        }
    }
}
