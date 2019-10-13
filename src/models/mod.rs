use libloading::Library as Dll;
use serde::{Deserialize, Serialize};

use database::{Count, Query, Queue};

use kpal_plugin::Value;

pub mod database;

#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(tag = "variant")]
pub enum Attribute {
    #[serde(rename(serialize = "integer", deserialize = "integer"))]
    Int { id: usize, name: String, value: i64 },

    #[serde(rename(serialize = "float", deserialize = "float"))]
    Float { id: usize, name: String, value: f64 },
}

impl Attribute {
    /// Converts a peripheral Value into an Attribute.
    ///
    /// This function makes it easier to convert Values, which are returned from the Peripheral's
    /// plugin API, to Attributes, which are passed across the REST API.
    ///
    /// # Arguments
    ///
    /// * `value` The value to assign to the new attribute
    /// * `id` The numeric ID of the attribute
    /// * `name` The attribute's name
    pub fn from(value: Value, id: usize, name: String) -> Attribute {
        match value {
            Value::Int(value) => Attribute::Int {
                id: id,
                name: name,
                value: value,
            },
            Value::Float(value) => Attribute::Float {
                id: id,
                name: name,
                value: value,
            },
        }
    }

    pub fn id(&self) -> usize {
        match self {
            Attribute::Int { id, .. } => *id,
            Attribute::Float { id, .. } => *id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Attribute::Int { name, .. } => name,
            Attribute::Float { name, .. } => name,
        }
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
                Attribute::Float {
                    id: id1,
                    name: name1,
                    value: value1,
                },
                Attribute::Float {
                    id: id2,
                    name: name2,
                    value: value2,
                },
            ) => id1 == id2 && name1 == name2 && value1 == value2,
            (_, _) => false,
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

impl Library {
    pub fn new(id: usize, name: String, library: Option<Dll>) -> Library {
        Library { id, name, library }
    }

    pub fn dll(&self) -> &Option<Dll> {
        &self.library
    }
}

impl Query for Library {
    fn id(&self) -> usize {
        self.id
    }

    fn key() -> &'static str {
        "libraries"
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
}

impl Count for Library {}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Peripheral {
    library_id: usize,
    name: String,

    #[serde(default)]
    attributes: Vec<Attribute>,

    #[serde(default)]
    id: usize,
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

    pub fn set_attribute_from_value(&mut self, id: usize, value: Value) {
        let attribute = self.attributes.get_mut(id).unwrap();
        *attribute = Attribute::from(value, id, attribute.name().to_owned());
    }
}

impl Query for Peripheral {
    fn id(&self) -> usize {
        self.id
    }

    fn key() -> &'static str {
        "peripherals"
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
}

impl Count for Peripheral {}

impl Queue for Peripheral {}

#[cfg(test)]
mod tests {
    use super::*;

    use kpal_plugin::Value;

    #[test]
    fn test_attribute_from() {
        let (id, name) = (0, String::from("foo"));
        let cases = vec![
            (
                Value::Int(42),
                Attribute::Int {
                    id: id,
                    name: name.clone(),
                    value: 42,
                },
            ),
            (
                Value::Float(42.42),
                Attribute::Float {
                    id: id,
                    name: name.clone(),
                    value: 42.42,
                },
            ),
        ];

        for (value, attr) in cases {
            let converted_attr = Attribute::from(value, id, name.clone());
            assert_eq!(attr, converted_attr);
        }
    }
}
