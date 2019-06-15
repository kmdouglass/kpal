use libloading::Library as Dll;

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
