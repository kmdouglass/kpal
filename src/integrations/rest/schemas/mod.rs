//! REST schemas convert user input data in JSON format into KPAL models and model builders.
//!
//! Data types in this library follow the format
//! `<MODEL><SUB_MODEL>[<SUB_MODEL>...]<CRUD>[Response]`, where
//!
//! * `MODEL` is the name of a KPAL model
//! * `SUB_MODEL` is the name of a nested model inside of `MODEL`
//! * `CRUD` is any combination of `Create`, `Read`, `Update`, or `Delete`
//! * `Response` is optional; if present, it indicates a response to a query. It is used to
//! disambiguate input data from returned data when necessary.
//!
//! Also defined in this module are `From<T>` trait implementations for the models and/or model
//! builders that correspond to integration types `T`.
mod errors;

use std::{
    convert::{TryFrom, TryInto},
    ffi::CString,
};

use serde::{Deserialize, Serialize};

use crate::{
    constants::BASE_URL_PATH,
    models::{Attribute, AttributeBuilder, Library, Model, Peripheral, PeripheralBuilder, Value},
};

pub use errors::SchemaError;

/// Data returned when a Peripheral Attribute is read.
#[derive(Debug, Serialize)]
pub struct AttributeRead {
    id: usize,
    name: String,
    value: ValueReadUpdate,
}

impl TryFrom<Attribute> for AttributeRead {
    type Error = SchemaError;

    fn try_from(attr: Attribute) -> Result<AttributeRead, Self::Error> {
        Ok(AttributeRead {
            id: attr.id(),
            name: attr.name().to_owned(),
            value: attr.value().clone().try_into()?,
        })
    }
}

/// Data returned in a request for a Library Attribute.
#[derive(Debug, Serialize)]
pub struct LibraryAttributeRead {
    id: usize,
    name: String,
    pre_init: bool,
    value: ValueReadUpdate,
}

impl TryFrom<Attribute> for LibraryAttributeRead {
    type Error = SchemaError;

    fn try_from(attr: Attribute) -> Result<LibraryAttributeRead, Self::Error> {
        Ok(LibraryAttributeRead {
            id: attr.id(),
            name: attr.name().to_owned(),
            pre_init: attr.pre_init(),
            value: attr.value().clone().try_into()?,
        })
    }
}

/// Data returned in a request for a Library or Libraries.
#[derive(Debug, Serialize)]
pub struct LibraryRead {
    attributes: Vec<LibraryAttributeRead>,
    id: usize,
    name: String,
}

impl TryFrom<Library> for LibraryRead {
    type Error = SchemaError;

    fn try_from(lib: Library) -> Result<LibraryRead, Self::Error> {
        let attrs: Vec<LibraryAttributeRead> = lib
            .attributes()
            .iter()
            .map(|(_, attr)| attr.clone())
            .map(|attr| attr.try_into())
            .collect::<Result<Vec<LibraryAttributeRead>, SchemaError>>()?;

        Ok(LibraryRead {
            attributes: attrs,
            id: lib.id(),
            name: lib.name().to_owned(),
        })
    }
}

/// Data that is used to create a new peripheral attribute.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum PeripheralAttributeCreate {
    #[serde(rename(serialize = "double", deserialize = "double"))]
    Double { id: usize, value: f64 },

    #[serde(rename(serialize = "integer", deserialize = "integer"))]
    Int { id: usize, value: i32 },

    #[serde(rename(serialize = "string", deserialize = "string"))]
    String { id: usize, value: String },

    #[serde(rename(serialize = "unsigned_integer", deserialize = "unsigned_integer"))]
    Uint { id: usize, value: u32 },
}

impl TryFrom<PeripheralAttributeCreate> for AttributeBuilder {
    type Error = SchemaError;

    fn try_from(data: PeripheralAttributeCreate) -> Result<AttributeBuilder, Self::Error> {
        use PeripheralAttributeCreate::*;

        let (id, value) = match data {
            Double { id, value } => (id, Value::Double { value }),
            Int { id, value } => (id, Value::Int { value }),
            String { id, value } => (
                id,
                Value::String {
                    value: CString::new(value)?,
                },
            ),
            Uint { id, value } => (id, Value::Uint { value }),
        };

        Ok(AttributeBuilder::new(id, value))
    }
}

/// Input data that is used to create a new peripheral.
#[derive(Debug, Deserialize)]
pub struct PeripheralCreate {
    attributes: Option<Vec<PeripheralAttributeCreate>>,
    library_id: usize,
    name: String,
}

impl TryFrom<PeripheralCreate> for PeripheralBuilder {
    type Error = SchemaError;

    fn try_from(data: PeripheralCreate) -> Result<PeripheralBuilder, Self::Error> {
        let mut builder = PeripheralBuilder::new(data.library_id, data.name);

        if let Some(attrs) = data.attributes {
            for attr in attrs {
                let attr_builder = AttributeBuilder::try_from(attr)?;
                builder = builder.set_attribute_builder(attr_builder);
            }
        };

        Ok(builder)
    }
}

/// Data returned when a Peripheral Attribute is read.
#[derive(Debug, Serialize)]
pub struct PeripheralAttributeRead {
    link: String,
}

impl From<Attribute> for PeripheralAttributeRead {
    fn from(attr: Attribute) -> PeripheralAttributeRead {
        let link = format!("{}/{}", Attribute::key(), attr.id());
        PeripheralAttributeRead { link }
    }
}

/// Data returned in a response to a request that resulted in the creation of a Peripheral.
#[derive(Debug, Serialize)]
pub struct PeripheralCreateResponse {
    pub message: String,
}

/// Data returned when a Peripheral is read.
#[derive(Debug, Serialize)]
pub struct PeripheralRead {
    attributes: Vec<PeripheralAttributeRead>,
    id: usize,
    library_id: usize,
    name: String,
}

impl From<Peripheral> for PeripheralRead {
    fn from(periph: Peripheral) -> PeripheralRead {
        let attrs: Vec<PeripheralAttributeRead> = periph
            .attributes()
            .iter()
            .map(|(_, attr)| attr.clone().into())
            .map(|mut attr: PeripheralAttributeRead| {
                attr.link = format!(
                    "{}/{}/{}/{}",
                    BASE_URL_PATH,
                    Peripheral::key(),
                    periph.id(),
                    attr.link
                );
                attr
            })
            .collect();

        PeripheralRead {
            attributes: attrs,
            id: periph.id(),
            library_id: periph.library_id(),
            name: periph.name().to_owned(),
        }
    }
}

/// Data returned in a request for a Value or used to update an attribute's value.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum ValueReadUpdate {
    #[serde(rename(deserialize = "double", serialize = "double"))]
    Double(f64),

    #[serde(rename(deserialize = "integer", serialize = "integer"))]
    Int(i32),

    #[serde(rename(deserialize = "string", serialize = "string"))]
    String(String),

    #[serde(rename(deserialize = "unsigned_integer", serialize = "unsigned_integer"))]
    Uint(u32),
}

impl TryFrom<Value> for ValueReadUpdate {
    type Error = SchemaError;

    fn try_from(value: Value) -> Result<ValueReadUpdate, Self::Error> {
        let value = match value {
            Value::Int { value, .. } => ValueReadUpdate::Int(value),
            Value::Double { value, .. } => ValueReadUpdate::Double(value),
            Value::String { value, .. } => {
                let string = CString::new(value)?.into_string()?;
                ValueReadUpdate::String(string)
            }
            Value::Uint { value, .. } => ValueReadUpdate::Uint(value),
        };

        Ok(value)
    }
}

impl TryFrom<ValueReadUpdate> for Value {
    type Error = SchemaError;

    fn try_from(data: ValueReadUpdate) -> Result<Value, Self::Error> {
        use ValueReadUpdate::*;

        let value = match data {
            Double(value) => Value::Double { value },
            Int(value) => Value::Int { value },
            String(value) => Value::String {
                value: CString::new(value)?,
            },
            Uint(value) => Value::Uint { value },
        };

        Ok(value)
    }
}
