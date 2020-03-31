//! Models represent the core abstractions of KPAL.
//!
//! Model instances correspond directly to objects in the User API, e.g.
//!
//! - peripherals
//! - attributes
//! - values
//! - libraries
mod errors;

use std::{
    collections::BTreeMap,
    ffi::{CStr, CString},
    slice,
};

use libloading::Library as Dll;

use kpal_plugin::Val as PluginValue;

pub use errors::ModelError;

/// A model represents one of the system's core abstractions.
pub trait Model {
    /// Returns the ID of the Model instance.
    fn id(&self) -> usize;

    /// Returns the key of the Model instance. Keys are used to give names to Models when building
    /// URLs or URIs.
    fn key() -> &'static str;
}

/// Attributes represent part of the entire state of a peripheral.
///
/// Each attribute is owned by one and only one peripheral. Its ID is unique within that peripheral
/// only.
#[derive(Clone, Debug)]
pub struct Attribute {
    /// The ID of the Attribute
    id: usize,

    /// The name of the Attribute
    name: String,

    /// Whether the attribute's default value may be overridden when the plugin is initialized
    pre_init: bool,

    /// The value of the Attribute
    value: Value,
}

impl Attribute {
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
    ) -> Result<Attribute, ModelError> {
        match value {
            PluginValue::Int(value) => Ok(Attribute {
                id,
                name,
                pre_init,
                value: Value::Int { value },
            }),
            PluginValue::Double(value) => Ok(Attribute {
                id,
                name,
                pre_init,
                value: Value::Double { value },
            }),
            PluginValue::String(p_value, length) => {
                let value = unsafe {
                    let slice = slice::from_raw_parts(p_value, length);
                    let string = CStr::from_bytes_with_nul(slice)?.to_str()?;
                    CString::new(string.to_owned())?
                };
                Ok(Attribute {
                    id,
                    name,
                    pre_init,
                    value: Value::String { value },
                })
            }
            PluginValue::Uint(value) => Ok(Attribute {
                id,
                name,
                pre_init,
                value: Value::Uint { value },
            }),
        }
    }

    /// Returns the name of an attribute.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Indicates whether an Attribute's value may be modified before peripheral initialization.
    pub fn pre_init(&self) -> bool {
        self.pre_init
    }

    /// Returns a new value instance that is created from an attribute.
    pub fn to_value(&self) -> Result<Value, ModelError> {
        let value = match &self.value {
            Value::Int { value, .. } => Value::Int { value: *value },
            Value::Double { value, .. } => Value::Double { value: *value },
            Value::String { value, .. } => {
                let c_string = CString::new(value.clone())?;
                Value::String { value: c_string }
            }
            Value::Uint { value, .. } => Value::Uint { value: *value },
        };

        Ok(value)
    }

    /// Returns a reference to the Attribute's value.
    pub fn value(&self) -> &Value {
        &self.value
    }
}

impl Model for Attribute {
    fn id(&self) -> usize {
        self.id
    }

    fn key() -> &'static str {
        "attributes"
    }
}

impl Eq for Attribute {}

impl PartialEq for Attribute {
    fn eq(&self, other: &Attribute) -> bool {
        self.id == other.id && self.name == other.name
    }
}

/// AttributeBuilders are used to initialize parts of new Attributes at different points in time.
///
/// AttributeBuilders allow the daemon to build new Attribute instances explicitly and sequentially
/// by setting values for an attribute's fields prior to initialization. When the Attribute is
/// ready to be intialized, the `build` method is called.
#[derive(Debug)]
pub struct AttributeBuilder {
    /// The ID of the Attribute
    id: usize,

    /// The name of the Attribute
    name: Option<String>,

    /// Whether the Attribute's default value may be overridden when the plugin is initialized
    pre_init: Option<bool>,

    /// The value of the Attribute
    value: Value,
}

impl AttributeBuilder {
    /// Creates a new instance of an AttributeBuilder
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the AttributeBuilder
    /// * `value` - The current value of the AttributeBuilder
    pub fn new(id: usize, value: Value) -> AttributeBuilder {
        AttributeBuilder {
            id,
            name: None,
            pre_init: None,
            value,
        }
    }

    /// Initializes a new Attribute instance from the builder.
    ///
    /// This method will consume the builder.
    pub fn build(self) -> Result<Attribute, ModelError> {
        Ok(Attribute {
            id: self.id,
            name: self.name.ok_or(ModelError::BuilderNotInitializedError)?,
            pre_init: self
                .pre_init
                .ok_or(ModelError::BuilderNotInitializedError)?,
            value: self.value,
        })
    }

    /// Returns the ID of the Attribute builder.
    pub fn id(&self) -> &usize {
        &self.id
    }

    /// Sets the name of the AttributeBuilder.
    ///
    /// # Arguments
    ///
    /// * `name` - The new name of the AttributeBuilder
    pub fn set_name(mut self, name: String) -> AttributeBuilder {
        self.name = Some(name);
        self
    }

    /// Sets the pre-init value of the AttributeBuilder
    ///
    /// # Arguments
    ///
    /// * `pre_init` - Whether the Attribute value can be set before the plugin is initialized
    pub fn set_pre_init(mut self, pre_init: bool) -> AttributeBuilder {
        self.pre_init = Some(pre_init);
        self
    }
}

/// A Library represents an interface to a plugin.
///
/// KPAL interfaces with plugins through library files. Libraries provide implementations of the
/// plugin API that is specific to each plugin.
///
/// A library file is the shared library that resides on the files system and that is pointed to by
/// this Model.
#[derive(Debug)]
pub struct Library {
    /// The plugin attributes that are defined by this Library.
    attributes: BTreeMap<usize, Attribute>,

    /// The ID of the Library.
    id: usize,

    /// A reference to the underlying shared library.
    library: Option<Dll>,

    /// The name of the library.
    name: String,
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
    /// Creates a new Library instance.
    ///
    /// Libraries contain an instance of a shared library object. This object provides an interface
    /// between Rust and the library file.
    ///
    /// # Arguments
    ///
    /// * `id` - The numeric ID of the attribute
    /// * `name` - The attribute's name
    /// * `library` The shared library that is used to manipulate the plugin
    pub fn new(id: usize, name: String, library: Option<Dll>) -> Library {
        let attributes: BTreeMap<usize, Attribute> = BTreeMap::new();
        Library {
            id,
            name,
            attributes,
            library,
        }
    }

    /// Returns the shared library instance.
    pub fn dll(&self) -> &Option<Dll> {
        &self.library
    }

    /// Returns the collection of attributes provided by the plugin library.
    pub fn attributes(&self) -> &BTreeMap<usize, Attribute> {
        &self.attributes
    }

    /// Returns the name of the Library.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Allows a Library's attributes to be set.
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

/// A Peripheral represents a single device or system controlled by KPAL.
///
/// A Peripheral is an interface to a Plugin. A plugin reprensts the actual device or system,
/// whereas a Peripheral is what the users sees and how she/he controls it.
#[derive(Clone, Debug)]
pub struct Peripheral {
    attributes: BTreeMap<usize, Attribute>,
    id: usize,
    library_id: usize,
    name: String,
}

impl Peripheral {
    /// Returns the collection of peripheral attributes.
    pub fn attributes(&self) -> &BTreeMap<usize, Attribute> {
        &self.attributes
    }

    /// Returns the ID of the Peripheral.
    pub fn library_id(&self) -> usize {
        self.library_id
    }

    /// Returns the name of the Peripheral.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the value of an Attribute to the value contained in a Value instance from a plugin.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the attribute to set
    /// * `value` - The Value instance from a plugin
    pub fn set_attribute_from_value(
        &mut self,
        id: usize,
        value: PluginValue,
    ) -> Result<(), ModelError> {
        let attribute = self.attributes.get_mut(&id).unwrap();
        *attribute = Attribute::new(value, id, attribute.name().to_owned(), attribute.pre_init())?;
        Ok(())
    }

    /// Sets the ID of the Peripheral.
    ///
    /// # Arguments
    ///
    /// * `id` - The new Peripheral ID.
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

/// PeripheralBuilders are used to initialize parts of new Peripherals at different points in time.
///
/// PeripheralBuilders allow the daemon to build new Peripheral instances explicitly and
/// sequentially by setting values for an peripheral's fields prior to initialization. When the
/// Peripheral is ready to be intialized, the `build` method is called.
pub struct PeripheralBuilder {
    /// The collection of Attributes that will belong to the eventual Peripheral instance.
    attributes: BTreeMap<usize, Attribute>,

    /// A collection of partially-initialized Attributes. This collection must be empty before the
    /// new Peripheral is created.
    attribute_builders: BTreeMap<usize, AttributeBuilder>,

    /// The ID of the PeripheralBuilder.
    id: Option<usize>,

    /// The ID of the plugin library that is used to control this Peripheral.
    library_id: usize,

    /// The name of the PeripheralBuilder.
    name: String,
}

impl PeripheralBuilder {
    pub fn new(library_id: usize, name: String) -> PeripheralBuilder {
        PeripheralBuilder {
            attributes: BTreeMap::new(),
            attribute_builders: BTreeMap::new(),
            id: None,
            library_id,
            name,
        }
    }

    /// Initializes a new peripheral instance from the builder.
    ///
    /// This method will consume the builder.
    pub fn build(self) -> Result<Peripheral, ModelError> {
        if !self.attribute_builders.is_empty() {
            return Err(ModelError::BuilderNotInitializedError);
        }

        Ok(Peripheral {
            attributes: self.attributes,
            id: self.id.ok_or(ModelError::BuilderNotInitializedError)?,
            library_id: self.library_id,
            name: self.name,
        })
    }

    /// Returns a single attribute from the builder.
    pub fn attribute(&self, id: usize) -> Option<&Attribute> {
        self.attributes.get(&id)
    }

    /// Returns all attributes of the builder.
    pub fn attributes(&self) -> &BTreeMap<usize, Attribute> {
        &self.attributes
    }

    /// Returns an owned instance of the AttributeBuilder with the given ID.
    ///
    /// Note that this will remove the AttributeBuilder from the collection that is owned by
    /// instances of the PeripheralBuilder struct. Calling this method on all AttributeBuilders
    /// will empty the collection.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the AttributeBuilder to retrieve
    pub fn attribute_builder(&mut self, id: usize) -> Option<AttributeBuilder> {
        self.attribute_builders.remove(&id)
    }

    /// Returns the library ID of the AttributeBuilder
    pub fn library_id(&self) -> &usize {
        &self.library_id
    }

    /// Inserts an Attribute into the collection of Attributes owned by this builder.
    ///
    /// # Arguments
    ///
    /// * `attr` - The Attribute to insert into this builder's collection
    pub fn set_attribute(mut self, attr: Attribute) -> PeripheralBuilder {
        let id = attr.id();
        self.attributes.insert(id, attr);
        self
    }

    /// Inserts an AttributeBuilder into the collection of AttributeBuilders.
    ///
    /// # Arguments
    ///
    /// * `attr` - The AttributeBuilder to insert into this builder's collection
    pub fn set_attribute_builder(mut self, builder: AttributeBuilder) -> PeripheralBuilder {
        let id = builder.id();
        self.attribute_builders.insert(*id, builder);
        self
    }

    /// Sets the ID of the PeripheralBuilder.
    ///
    /// # Arguments
    ///
    /// *`id` - The new ID of the PeripheralBuilder
    pub fn set_id(mut self, id: usize) -> PeripheralBuilder {
        self.id = Some(id);
        self
    }
}

#[derive(Clone, Debug)]
/// A Value represents the current value of an Attribute.
pub enum Value {
    Int { value: i32 },
    Double { value: f64 },
    String { value: CString },
    Uint { value: u32 },
}

impl Value {
    /// Returns a Val (a reference-like Value object) from a Value.
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
    fn test_peripheral_set_attribute_from_value() {
        let mut context = set_up();
        let new_value = PluginValue::Double(PI);
        let new_attr = Attribute {
            id: context.float_id,
            name: context.name.clone(),
            pre_init: context.pre_init,
            value: Value::Double { value: PI },
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
            Attribute {
                id: int_id,
                name: name.clone(),
                pre_init,
                value: Value::Int { value: int_value },
            },
        );
        attributes.insert(
            float_id,
            Attribute {
                id: float_id,
                name: name.clone(),
                pre_init,
                value: Value::Double { value: float_value },
            },
        );

        let peripheral = Peripheral {
            library_id,
            name: name.clone(),
            attributes: attributes.clone(),
            id: 0,
        };

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
