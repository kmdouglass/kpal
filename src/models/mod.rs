use libloading::Library as Dll;
use serde::{Deserialize, Serialize};

use database::{Count, Query, Queue};

pub mod database;

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "variant")]
pub enum Attribute {
    #[serde(rename(serialize = "integer", deserialize = "integer"))]
    Int { id: usize, name: String, value: i64 },

    #[serde(rename(serialize = "float", deserialize = "float"))]
    Float { id: usize, name: String, value: f64 },
}

#[derive(Deserialize, Serialize)]
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

#[derive(Clone, Deserialize, Serialize)]
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
