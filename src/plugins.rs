use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use libloading::Symbol;

use kpal_peripheral::Peripheral as Plugin;
use kpal_peripheral::PeripheralNew;

use crate::models::{Library, Peripheral};

/// A thread safe version of a [Library](../models/struct.Library.html) instance.
///
/// This is a convenience type for sharing a single a Library instance between multiple
/// threads. Due to its use of a Mutex, different peripherals that use the same library will not
/// make function calls from the library in a deterministic order.
pub type TSLibrary = Arc<Mutex<Library>>;

pub fn init(
    _peripheral: &mut Peripheral,
    _db: &redis::Connection,
    lib: TSLibrary,
) -> std::result::Result<(), PluginInitError> {
    thread::spawn(move || -> Result<(), PeripheralThreadError> {
        let peripheral_p: *mut Plugin =
            unsafe { peripheral_new(lib).map_err(|_| PeripheralThreadError {})? };

        loop {
            println!("inside plugin loop with pointer: {:?}", peripheral_p);
            thread::sleep(Duration::from_secs(5));
        }
    });

    Ok(())
}

unsafe fn peripheral_new(lib: TSLibrary) -> Result<*mut Plugin, PeripheralNewError> {
    let lib = lib.lock().map_err(|_| PeripheralNewError {})?;

    let dll = lib.dll().as_ref().ok_or(PeripheralNewError {})?;

    let init: Symbol<PeripheralNew> = dll
        .get(b"peripheral_new\0")
        .map_err(|_| PeripheralNewError {})?;

    Ok(init())
}

#[derive(Debug)]
pub struct PluginInitError {
    side: Box<dyn Error>,
}

impl Error for PluginInitError {
    fn description(&self) -> &str {
        "Failed to initialize the plugin"
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.side)
    }
}

impl fmt::Display for PluginInitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PluginInitError {{ Cause: {} }}", &*self.side)
    }
}

#[derive(Debug)]
pub struct PeripheralThreadError {}

impl Error for PeripheralThreadError {
    fn description(&self) -> &str {
        "The peripheral thread failed"
    }
}

impl fmt::Display for PeripheralThreadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The peripheral thread failed")
    }
}

#[derive(Debug)]
pub struct PeripheralNewError {}

impl Error for PeripheralNewError {
    fn description(&self) -> &str {
        "Failed to fetch a symbol from the library"
    }
}

impl fmt::Display for PeripheralNewError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not create a new peripheral")
    }
}
