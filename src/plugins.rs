use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use libloading::Symbol;

use kpal_peripheral::Peripheral as Plugin;
use kpal_peripheral::{PeripheralNew, VTable, VTableNew};

use crate::models::{Library, Peripheral};

/// A thread safe version of a [Library](../models/struct.Library.html) instance.
///
/// This is a convenience type for sharing a single a Library instance between multiple
/// threads. Due to its use of a Mutex, different peripherals that use the same library will not
/// make function calls from the library in a deterministic order.
pub type TSLibrary = Arc<Mutex<Library>>;

/// A PluginManager contains the necessary data to work with a Plugin across the FFI boundary.
///
/// This struct holds a raw pointer to aperipheral that is created by the Peripheral library. In
/// addition it contains the vtable of function pointers defined by the C API and implemented
/// within the Peripheral library.
///
/// The PluginManager implements the `Send` trait because after creation the PluginManager is moved
/// into the thread that is dedicated to the peripheral that it manages. Once it is moved, it will
/// only ever be owned by this thread by design.
#[derive(Debug)]
struct PluginManager {
    object_p: *mut Plugin,
    vtable: VTable,
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        (self.vtable.peripheral_free)(self.object_p);
    }
}

unsafe impl Send for PluginManager {}

pub fn init(
    _peripheral: &mut Peripheral,
    _db: &redis::Connection,
    lib: TSLibrary,
) -> std::result::Result<(), PluginInitError> {
    let peripheral_p: *mut Plugin =
        unsafe { peripheral_new(lib.clone()).map_err(|e| PluginInitError { side: Box::new(e) })? };

    let vtable: VTable =
        unsafe { peripheral_vtable(lib).map_err(|e| PluginInitError { side: Box::new(e) })? };

    let plugin = PluginManager {
        object_p: peripheral_p,
        vtable: vtable,
    };

    thread::spawn(move || -> Result<(), PeripheralThreadError> {
        loop {
            println!("inside plugin loop with plugin: {:?}", plugin);
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

unsafe fn peripheral_vtable(lib: TSLibrary) -> Result<VTable, VTableError> {
    let lib = lib.lock().map_err(|_| VTableError {})?;

    let dll = lib.dll().as_ref().ok_or(VTableError {})?;

    let vtable: Symbol<VTableNew> = dll
        .get(b"peripheral_vtable\0")
        .map_err(|_| VTableError {})?;

    Ok(vtable())
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

#[derive(Debug)]
pub struct VTableError {}

impl Error for VTableError {
    fn description(&self) -> &str {
        "Failed to fetch the vtable from the library"
    }
}

impl fmt::Display for VTableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not create a new vtable")
    }
}
