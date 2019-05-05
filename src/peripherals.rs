use libloading::Library;

/// A peripheral is a device that is controlled by the deamon.
///
/// The interface to the peripheral is a dynamically loaded library and a C API.
pub struct Peripheral {
    lib: Library,
}

impl Peripheral {
    /// Creates a new peripheral instance.
    ///
    /// # Arguments
    ///
    /// * `lib` - The dynamic library associated with this peripheral.
    pub fn new(lib: Library) -> Peripheral {
        Peripheral { lib: lib }
    }
}
