mod errors;

use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    slice,
};

use libloading::Library as Dll;
use serde::{Deserialize, Serialize};

use kpal_plugin::Val as PluginValue;

pub use errors::*;

pub trait Model {
    fn id(&self) -> usize;

    fn key() -> &'static str;
}

#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(tag = "variant")]
pub enum Attribute {
    #[serde(rename(serialize = "integer", deserialize = "integer"))]
    Int { id: usize, name: String, value: i32 },

    #[serde(rename(serialize = "double", deserialize = "double"))]
    Double { id: usize, name: String, value: f64 },

    #[serde(rename(serialize = "string", deserialize = "string"))]
    String {
        id: usize,
        name: String,
        value: String,
    },
}

impl Attribute {
    pub fn name(&self) -> &str {
        match self {
            Attribute::Int { name, .. } => name,
            Attribute::Double { name, .. } => name,
            Attribute::String { name, .. } => name,
        }
    }

    /// Creates a new Attribute instance from a PluginValue.
    ///
    /// This function makes it easier to convert PluginValues, which are returned from the
    /// Peripheral's plugin API, to Attributes, which are passed across the REST API.
    ///
    /// # Arguments
    ///
    /// * `value` The value to assign to the new attribute
    /// * `id` The numeric ID of the attribute
    /// * `name` The attribute's name
    pub fn new(
        value: PluginValue,
        id: usize,
        name: String,
    ) -> Result<Attribute, ValueConversionError> {
        match value {
            PluginValue::Int(value) => Ok(Attribute::Int { id, name, value }),
            PluginValue::Double(value) => Ok(Attribute::Double { id, name, value }),
            PluginValue::String(p_value, length) => {
                let value = unsafe {
                    let slice = slice::from_raw_parts(p_value, length);
                    let string = CStr::from_bytes_with_nul(slice)?.to_str()?;
                    string.to_owned()
                };
                Ok(Attribute::String { id, name, value })
            }
        }
    }
}

impl Model for Attribute {
    fn id(&self) -> usize {
        match self {
            Attribute::Int { id, .. } => *id,
            Attribute::Double { id, .. } => *id,
            Attribute::String { id, .. } => *id,
        }
    }

    fn key() -> &'static str {
        "attributes"
    }
}

impl Eq for Attribute {}

impl PartialEq for Attribute {
    fn eq(&self, other: &Attribute) -> bool {
        match (self, other) {
            (
                Attribute::Int {
                    id: id1,
                    name: name1,
                    value: value1,
                },
                Attribute::Int {
                    id: id2,
                    name: name2,
                    value: value2,
                },
            ) => id1 == id2 && name1 == name2 && value1 == value2,
            (
                Attribute::Double {
                    id: id1,
                    name: name1,
                    value: value1,
                },
                Attribute::Double {
                    id: id2,
                    name: name2,
                    value: value2,
                },
            ) => id1 == id2 && name1 == name2 && value1 == value2,
            (_, _) => false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "variant")]
/// A Value represents the current value of an Attribute.
///
/// This enum is used to translate between values received from requests to update an attribute's
/// state and values understood by the plugin API.
pub enum Value {
    #[serde(rename(serialize = "integer", deserialize = "integer"))]
    Int { value: i32 },
    #[serde(rename(serialize = "double", deserialize = "double"))]
    Double { value: f64 },
    #[serde(rename(serialize = "string", deserialize = "string"))]
    String { value: CString },
}

impl Value {
    pub fn as_val(&self) -> PluginValue {
        match self {
            Value::Int { value } => PluginValue::Int(*value),
            Value::Double { value } => PluginValue::Double(*value),
            Value::String { value } => {
                let slice = value.as_bytes_with_nul();
                PluginValue::String(slice.as_ptr(), slice.len())
            }
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Library {
    id: usize,
    name: String,

    #[serde(skip)]
    library: Option<Dll>,
}

impl Clone for Library {
    /// Clones a library by ignoring any dynamic library owned by the model.
    fn clone(&self) -> Self {
        Library {
            id: self.id,
            library: None,
            name: self.name.clone(),
        }
    }
}

impl Library {
    pub fn new(id: usize, name: String, library: Option<Dll>) -> Library {
        Library { id, name, library }
    }

    pub fn dll(&self) -> &Option<Dll> {
        &self.library
    }
}

impl Model for Library {
    fn id(&self) -> usize {
        self.id
    }

    fn key() -> &'static str {
        "libraries"
    }
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Peripheral {
    library_id: usize,
    name: String,

    #[serde(skip)]
    attributes: Vec<Attribute>,

    #[serde(default)]
    id: usize,

    #[serde(default)]
    links: Vec<HashMap<String, String>>,
}

impl Peripheral {
    pub fn attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }

    pub fn library_id(&self) -> usize {
        self.library_id
    }

    pub fn set_attribute(&mut self, id: usize, attribute: Attribute) {
        match self.attributes.get_mut(id) {
            Some(old_attribute) => *old_attribute = attribute,
            None => {
                log::debug!("could not set attribute; index not valid: {}", id);
            }
        }
    }

    pub fn set_attributes(&mut self, attributes: Vec<Attribute>) {
        self.attributes = attributes;
    }

    pub fn set_attribute_from_value(
        &mut self,
        id: usize,
        value: PluginValue,
    ) -> Result<(), AttributeError> {
        let attribute = self.attributes.get_mut(id).unwrap();
        *attribute = Attribute::new(value, id, attribute.name().to_owned())?;
        Ok(())
    }

    pub fn set_attribute_links(&mut self) {
        let mut links = Vec::new();
        for attr in &self.attributes {
            let mut link = HashMap::new();
            link.insert(
                "href".to_string(),
                format!("/api/v0/peripherals/{}/attributes/{}", self.id, attr.id()),
            );
            links.push(link);
        }

        self.links = links;
    }

    pub fn set_id(&mut self, id: usize) {
        self.id = id;
    }
}

impl Model for Peripheral {
    fn id(&self) -> usize {
        self.id
    }

    fn key() -> &'static str {
        "peripherals"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::f64::consts::PI;

    use kpal_plugin::Val as PluginValue;

    #[test]
    fn test_attribute_from() {
        let context = set_up();
        let values = vec![
            PluginValue::Int(context.int_value),
            PluginValue::Double(context.float_value),
        ];
        let cases = values.into_iter().zip(context.attributes);

        for (value, attr) in cases {
            let converted_attr = Attribute::new(value, context.id, context.name.clone()).unwrap();
            assert_eq!(attr, converted_attr);
        }
    }

    #[test]
    fn test_attribute_id() {
        let context = set_up();
        let names = vec![context.id, context.id];
        let cases = names.into_iter().zip(context.attributes);

        for case in cases {
            let (id, attr) = case;
            assert_eq!(id, attr.id());
        }
    }

    #[test]
    fn test_attribute_name() {
        let context = set_up();
        let names = vec![context.name.clone(), context.name.clone()];
        let cases = names.into_iter().zip(context.attributes);

        for case in cases {
            let (name, attr) = case;
            assert_eq!(name, attr.name());
        }
    }

    #[test]
    fn test_library_new() {
        let context = set_up();
        let library = Library::new(context.id, context.name.clone(), None);

        assert_eq!(library.id, context.id);
        assert_eq!(library.name, context.name);
        assert!(library.library.is_none());
    }

    #[test]
    fn test_library_dll() {
        let context = set_up();
        let library = Library {
            id: context.id,
            name: context.name,
            library: None,
        };

        assert!(library.dll().is_none());
    }

    #[test]
    fn test_peripheral_attributes() {
        let context = set_up();
        assert_eq!(*context.peripheral.attributes(), context.attributes);
    }

    #[test]
    fn test_peripheral_library_id() {
        let context = set_up();
        assert_eq!(context.peripheral.library_id(), context.library_id);
    }

    #[test]
    fn test_peripheral_set_attribute() {
        let mut context = set_up();
        let new_attr = Attribute::Double {
            id: context.id,
            name: context.name,
            value: PI,
        };

        assert_ne!(context.peripheral.attributes[0], new_attr);

        context.peripheral.set_attribute(0, new_attr.clone());
        assert_eq!(context.peripheral.attributes[0], new_attr);
    }

    #[test]
    fn test_peripheral_set_attributes() {
        let mut context = set_up();
        let new_attr = Attribute::Double {
            id: context.id,
            name: context.name.clone(),
            value: PI,
        };

        for attr in context.peripheral.attributes.clone() {
            assert_ne!(attr, new_attr);
        }

        context.peripheral.set_attributes(vec![new_attr.clone()]);
        for attr in context.peripheral.attributes {
            assert_eq!(attr, new_attr);
        }
    }

    #[test]
    fn test_peripheral_set_attribute_from_value() {
        let mut context = set_up();
        let new_value = PluginValue::Double(PI);
        let new_attr = Attribute::Double {
            id: context.id,
            name: context.name.clone(),
            value: PI,
        };

        assert_ne!(context.peripheral.attributes[0], new_attr);

        context
            .peripheral
            .set_attribute_from_value(0, new_value)
            .unwrap();
        assert_eq!(context.peripheral.attributes[0], new_attr);
    }

    struct Context {
        attributes: Vec<Attribute>,
        float_value: f64,
        id: usize,
        int_value: i32,
        library_id: usize,
        name: String,
        peripheral: Peripheral,
    }

    fn set_up() -> Context {
        let (id, name, int_value, float_value) = (0, String::from("foo"), 42, 42.42);
        let library_id = 1;
        let attributes = vec![
            Attribute::Int {
                id: id,
                name: name.clone(),
                value: int_value,
            },
            Attribute::Double {
                id: id,
                name: name.clone(),
                value: float_value,
            },
        ];

        let mut peripheral = Peripheral {
            library_id,
            name: name.clone(),
            attributes: attributes.clone(),
            id,
            links: Vec::new(),
        };
        peripheral.set_attribute_links();

        Context {
            attributes,
            float_value,
            id,
            int_value,
            library_id,
            name,
            peripheral,
        }
    }
}
