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
