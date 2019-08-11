use libloading::Library as Dll;
use serde::{Deserialize, Serialize};

use database::{Count, Query};

pub mod database;

#[derive(Deserialize, Serialize)]
pub struct Library {
    pub id: usize,
    pub name: String,

    // TODO Change `default = "none"` to `default` because Option is Default
    #[serde(skip, default = "none")]
    library: Option<Dll>,
}

impl Library {
    pub fn new(id: usize, name: String, library: Option<Dll>) -> Library {
        Library { id, name, library }
    }
}

impl Query for Library {
    fn id(&self) -> usize {
        self.id
    }

    fn key() -> &'static str {
        "libraries"
    }
}

impl Count for Library {}

#[derive(Deserialize, Serialize)]
pub struct Peripheral {
    #[serde(default)]
    pub id: usize,
    pub library_id: usize,
    pub name: String,
}

impl Query for Peripheral {
    fn id(&self) -> usize {
        self.id
    }

    fn key() -> &'static str {
        "peripherals"
    }
}

impl Count for Peripheral {}

fn none() -> Option<Dll> {
    None
}
