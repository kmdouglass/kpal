use std::boxed::Box;
use std::ffi::CString;
use std::ptr;

use libc::{c_char, size_t};

use kpal_peripheral::error::Error;
use kpal_peripheral::{KpalApiV0, Property, Value};

struct Basic {
    error: Error,
    props: Vec<Property>,
}

impl KpalApiV0<Basic> for Basic {
    fn kpal_api_new() -> *mut Basic {
        Box::into_raw(Box::new(Basic {
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
        }))
    }

    fn kpal_api_free(kpal_api: *mut Basic) {
        if kpal_api.is_null() {
            return;
        }
        unsafe {
            Box::from_raw(kpal_api);
        }
    }

    fn kpal_property_name(&self, id: usize) -> &str {
        self.props[id].name
    }

    fn kpal_property_value(&self, id: usize) -> &Value {
        &self.props[id].value
    }

    fn kpal_property_set_value(&mut self, id: usize, value: Value) {
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

// TODO Generate the C-bindings in a macro
#[no_mangle]
extern "C" fn kpal_api_new() -> *mut Basic {
    Basic::kpal_api_new()
}

#[no_mangle]
extern "C" fn kpal_api_free(kpal_api: *mut Basic) {
    Basic::kpal_api_free(kpal_api);
}

#[no_mangle]
extern "C" fn kpal_error(kpal_api: *mut Basic) -> *const c_char {
    unsafe {
        let msg = (*kpal_api).error.query();
        match msg {
            Some(msg) => msg.as_ptr(),
            None => ptr::null(),
        }
    }
}

#[no_mangle]
extern "C" fn kpal_property_name(kpal_api: *const Basic, id: size_t) -> *const c_char {
    let api = unsafe {
        assert!(!kpal_api.is_null());
        &(*kpal_api)
    };
    CString::new(api.kpal_property_name(id))
        .expect("Error: CString::new()")
        .into_raw()
}

#[no_mangle]
extern "C" fn kpal_property_value(kpal_api: *const Basic, id: size_t) -> *const Value {
    let api = unsafe {
        assert!(!kpal_api.is_null());
        &(*kpal_api)
    };
    api.kpal_property_value(id)
}

#[no_mangle]
extern "C" fn kpal_property_set_value(kpal_api: *mut Basic, id: size_t, value: *const Value) {
    let (api, value) = unsafe {
        if kpal_api.is_null() || value.is_null() {
            return;
        }
        (&mut *kpal_api, *value)
    };
    api.kpal_property_set_value(id, value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_property_value() {
        let mut peripheral = unsafe { *Box::from_raw(Basic::kpal_api_new()) };
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
            peripheral.kpal_property_set_value(i, value);
            assert_eq!(
                value, peripheral.props[i].value,
                "Expected property value to be {:?} but it was {:?}",
                value, peripheral.props[i].value
            )
        }
    }

    #[test]
    fn set_property_wrong_variant() {
        let mut peripheral = unsafe { *Box::from_raw(Basic::kpal_api_new()) };
        let new_value = Value::_Float(42.0);

        peripheral.kpal_property_set_value(1, new_value);
        let error = peripheral.error.query();
        match error {
            Some(_msg) => (),
            None => panic!("Expected a triggered error state."),
        }
    }
}
