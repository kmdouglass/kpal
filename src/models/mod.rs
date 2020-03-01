mod errors;

use std::{
    collections::{BTreeMap, HashMap},
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

/// Attributes partially represent the complete state of a peripheral.
///
/// id and value are currenly the only fields that are deserialized because attributes are only
/// sent once during the lifetime of a peripheral before it is initialized. At this time, only id a
/// value are needed.
#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(tag = "variant")]
pub enum Attribute {
    #[serde(rename(serialize = "integer", deserialize = "integer"))]
    Int {
        id: usize,

        #[serde(default)]
        name: String,

        #[serde(default)]
        pre_init: bool,

        value: i32,
    },

    #[serde(rename(serialize = "double", deserialize = "double"))]
    Double {
        id: usize,

        #[serde(default)]
        name: String,

        #[serde(default)]
        pre_init: bool,

        value: f64,
    },

    #[serde(rename(serialize = "string", deserialize = "string"))]
    String {
        id: usize,

        #[serde(default)]
        name: String,

        #[serde(default)]
        pre_init: bool,

        value: String,
    },
    #[serde(rename(serialize = "unsigned_integer", deserialize = "unsigned_integer"))]
    Uint {
        id: usize,

        #[serde(default)]
        name: String,

        #[serde(default)]
        pre_init: bool,

        value: u32,
    },
}

impl Attribute {
    /// Returns the name of an attribute.
    pub fn name(&self) -> &str {
        match self {
            Attribute::Int { name, .. } => name,
            Attribute::Double { name, .. } => name,
            Attribute::String { name, .. } => name,
            Attribute::Uint { name, .. } => name,
        }
    }

    /// Indicates whether an attribute's value may be modified before peripheral initialization.
    pub fn pre_init(&self) -> bool {
        match self {
            Attribute::Int { pre_init, .. } => *pre_init,
            Attribute::Double { pre_init, .. } => *pre_init,
            Attribute::String { pre_init, .. } => *pre_init,
            Attribute::Uint { pre_init, .. } => *pre_init,
        }
    }

    /// Creates a new Attribute instance from a PluginValue.
    ///
    /// This function makes it easier to convert PluginValues, which are returned from the
    /// Peripheral's plugin API, to Attributes, which are passed across the REST API.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to assign to the new attribute
    /// * `id` - The numeric ID of the attribute
    /// * `name` - The attribute's name
    /// * `pre_init` - Detemines whether the attribute may be set before plugin initialization
    pub fn new(
        value: PluginValue,
        id: usize,
        name: String,
        pre_init: bool,
    ) -> Result<Attribute, ValueConversionError> {
        match value {
            PluginValue::Int(value) => Ok(Attribute::Int {
                id,
                name,
                pre_init,
                value,
            }),
            PluginValue::Double(value) => Ok(Attribute::Double {
                id,
                name,
                pre_init,
                value,
            }),
            PluginValue::String(p_value, length) => {
                let value = unsafe {
                    let slice = slice::from_raw_parts(p_value, length);
                    let string = CStr::from_bytes_with_nul(slice)?.to_str()?;
                    string.to_owned()
                };
                Ok(Attribute::String {
                    id,
                    name,
                    pre_init,
                    value,
                })
            }
            PluginValue::Uint(value) => Ok(Attribute::Uint {
                id,
                name,
                pre_init,
                value,
            }),
        }
    }

    /// Returns a new value instance that is created from an attribute.
    pub fn to_value(&self) -> Result<Value, ValueConversionError> {
        let value = match self {
            Attribute::Int { value, .. } => Value::Int { value: *value },
            Attribute::Double { value, .. } => Value::Double { value: *value },
            Attribute::String { value, .. } => {
                let c_string = CString::new(value.clone())?;
                Value::String { value: c_string }
            }
            Attribute::Uint { value, .. } => Value::Uint { value: *value },
        };

        Ok(value)
    }
}

impl Model for Attribute {
    fn id(&self) -> usize {
        match self {
            Attribute::Int { id, .. } => *id,
            Attribute::Double { id, .. } => *id,
            Attribute::String { id, .. } => *id,
            Attribute::Uint { id, .. } => *id,
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
                    value: value1,
                    ..
                },
                Attribute::Int {
                    id: id2,
                    value: value2,
                    ..
                },
            ) => id1 == id2 && value1 == value2,
            (
                Attribute::Double {
                    id: id1,
                    value: value1,
                    ..
                },
                Attribute::Double {
                    id: id2,
                    value: value2,
                    ..
                },
            ) => id1 == id2 && value1 == value2,
            (
                Attribute::String {
                    id: id1,
                    value: value1,
                    ..
                },
                Attribute::String {
                    id: id2,
                    value: value2,
                    ..
                },
            ) => id1 == id2 && value1 == value2,
            (
                Attribute::Uint {
                    id: id1,
                    value: value1,
                    ..
                },
                Attribute::Uint {
                    id: id2,
                    value: value2,
                    ..
                },
            ) => id1 == id2 && value1 == value2,
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
    #[serde(rename(serialize = "unsigned_integer", deserialize = "unsigned_integer"))]
    Uint { value: u32 },
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
            Value::Uint { value } => PluginValue::Uint(*value),
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Library {
    id: usize,
    name: String,
    attributes: BTreeMap<usize, Attribute>,

    #[serde(skip)]
    library: Option<Dll>,
}

impl Clone for Library {
    /// Clones a library by ignoring any dynamic library owned by the model.
    fn clone(&self) -> Self {
        Library {
            id: self.id,
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            library: None,
        }
    }
}

impl Library {
    pub fn new(id: usize, name: String, library: Option<Dll>) -> Library {
        let attributes: BTreeMap<usize, Attribute> = BTreeMap::new();
        Library {
            id,
            name,
            attributes,
            library,
        }
    }

    pub fn dll(&self) -> &Option<Dll> {
        &self.library
    }

    pub fn attributes(&self) -> &BTreeMap<usize, Attribute> {
        &self.attributes
    }

    pub fn set_attributes(&mut self, attributes: BTreeMap<usize, Attribute>) {
        self.attributes = attributes;
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

    #[serde(default, skip_serializing)]
    attributes: BTreeMap<usize, Attribute>,

    #[serde(default)]
    id: usize,

    #[serde(default, rename = "attributes")]
    links: Vec<HashMap<String, String>>,
}

impl Peripheral {
    pub fn attributes(&self) -> &BTreeMap<usize, Attribute> {
        &self.attributes
    }

    pub fn library_id(&self) -> usize {
        self.library_id
    }

    pub fn set_attribute(&mut self, id: usize, attribute: Attribute) {
        match self.attributes.get_mut(&id) {
            Some(old_attribute) => *old_attribute = attribute,
            None => {
                log::debug!("could not set attribute; index not valid: {}", id);
            }
        }
    }

    pub fn set_attributes(&mut self, attributes: BTreeMap<usize, Attribute>) {
        self.attributes = attributes;
    }

    pub fn set_attribute_from_value(
        &mut self,
        id: usize,
        value: PluginValue,
    ) -> Result<(), AttributeError> {
        let attribute = self.attributes.get_mut(&id).unwrap();
        *attribute = Attribute::new(value, id, attribute.name().to_owned(), attribute.pre_init())?;
        Ok(())
    }

    pub fn set_attribute_links(&mut self) {
        let mut links: Vec<HashMap<String, String>> = Vec::new();
        for attr in self.attributes.values() {
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
    fn test_attribute_new() {
        let context = set_up();
        let cases = vec![
            (
                PluginValue::Int(context.int_value),
                context.int_id,
                context.attributes.get(&context.int_id).unwrap(),
            ),
            (
                PluginValue::Double(context.float_value),
                context.float_id,
                context.attributes.get(&context.float_id).unwrap(),
            ),
        ];

        for (value, id, attr) in cases {
            let converted_attr =
                Attribute::new(value, id, context.name.clone(), context.pre_init).unwrap();
            assert_eq!(attr, &converted_attr);
        }
    }

    #[test]
    fn test_attribute_id() {
        let context = set_up();

        for (id, attr) in context.attributes {
            assert_eq!(id, attr.id());
        }
    }

    #[test]
    fn test_attribute_name() {
        let context = set_up();
        let cases = vec![
            (
                context.name.clone(),
                context.attributes.get(&context.int_id).unwrap(),
            ),
            (
                context.name.clone(),
                context.attributes.get(&context.float_id).unwrap(),
            ),
        ];

        for case in cases {
            let (name, attr) = case;
            assert_eq!(name, attr.name());
        }
    }

    #[test]
    fn test_library_new() {
        let context = set_up();
        let library = Library::new(0, context.name.clone(), None);

        assert_eq!(library.id, 0);
        assert_eq!(library.name, context.name);
        assert!(library.library.is_none());
    }

    #[test]
    fn test_library_dll() {
        let context = set_up();
        let library = Library {
            id: 0,
            name: context.name,
            attributes: context.attributes,
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
            id: context.float_id,
            name: context.name,
            pre_init: context.pre_init,
            value: PI,
        };

        assert_ne!(
            context
                .peripheral
                .attributes
                .get(&context.float_id)
                .unwrap(),
            &new_attr
        );

        context
            .peripheral
            .set_attribute(context.float_id, new_attr.clone());
        assert_eq!(
            context
                .peripheral
                .attributes
                .get(&context.float_id)
                .unwrap(),
            &new_attr
        );
    }

    #[test]
    fn test_peripheral_set_attributes() {
        let mut context = set_up();
        let new_attr = Attribute::Double {
            id: context.float_id,
            name: context.name.clone(),
            pre_init: context.pre_init,
            value: PI,
        };
        let mut new_attrs = BTreeMap::new();
        new_attrs.insert(context.float_id, new_attr.clone());

        for attr in context.peripheral.attributes.clone().values() {
            assert_ne!(attr, &new_attr);
        }

        context.peripheral.set_attributes(new_attrs);
        for (id, attr) in context.peripheral.attributes {
            assert_eq!(attr, new_attr);
            assert_eq!(context.float_id, id);
        }
    }

    #[test]
    fn test_peripheral_set_attribute_from_value() {
        let mut context = set_up();
        let new_value = PluginValue::Double(PI);
        let new_attr = Attribute::Double {
            id: context.float_id,
            name: context.name.clone(),
            pre_init: context.pre_init,
            value: PI,
        };

        assert_ne!(context.peripheral.attributes.get(&0).unwrap(), &new_attr);

        context
            .peripheral
            .set_attribute_from_value(context.float_id, new_value)
            .unwrap();
        assert_eq!(
            context
                .peripheral
                .attributes
                .get(&context.float_id)
                .unwrap(),
            &new_attr
        );
    }

    struct Context {
        attributes: BTreeMap<usize, Attribute>,
        float_id: usize,
        float_value: f64,
        int_id: usize,
        int_value: i32,
        library_id: usize,
        name: String,
        peripheral: Peripheral,
        pre_init: bool,
    }

    fn set_up() -> Context {
        let (name, int_value, float_value) = (String::from("foo"), 42, 42.42);
        let (int_id, float_id) = (0, 1);
        let library_id = 1;
        let pre_init = false;
        let mut attributes: BTreeMap<usize, Attribute> = BTreeMap::new();
        attributes.insert(
            int_id,
            Attribute::Int {
                id: int_id,
                name: name.clone(),
                pre_init,
                value: int_value,
            },
        );
        attributes.insert(
            float_id,
            Attribute::Double {
                id: float_id,
                name: name.clone(),
                pre_init,
                value: float_value,
            },
        );

        let mut peripheral = Peripheral {
            library_id,
            name: name.clone(),
            attributes: attributes.clone(),
            id: 0,
            links: Vec::new(),
        };
        peripheral.set_attribute_links();

        Context {
            attributes,
            float_id,
            float_value,
            int_id,
            int_value,
            library_id,
            name,
            peripheral,
            pre_init,
        }
    }
}
