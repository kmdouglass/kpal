use libloading::Library as Dll;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Library {
    pub id: usize,
    pub name: String,

    #[serde(skip, default = "none")]
    library: Option<Dll>,
}

impl Library {
    pub fn new(id: usize, name: String, library: Option<Dll>) -> Library {
        Library { id, name, library }
    }
}

fn none() -> Option<Dll> {
    None
}
