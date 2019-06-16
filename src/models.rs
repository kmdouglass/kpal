use libloading::Library as Dll;
use serde::ser::{Serialize, SerializeStruct, Serializer};

pub struct Library {
    pub id: usize,
    pub name: String,
    library: Dll,
}

impl Library {
    pub fn new(id: usize, name: String, library: Dll) -> Library {
        Library { id, name, library }
    }
}

impl Serialize for Library {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Library", 2)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.end()
    }
}
