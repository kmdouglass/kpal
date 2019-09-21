mod driver;
mod init;
mod scheduler;

use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};

use libloading::Symbol;

use kpal_peripheral::Peripheral as CPeripheral;
use kpal_peripheral::{PeripheralNew, VTable, VTableNew};

use crate::constants::SCHEDULER_SLEEP_DURATION;
use crate::models::Library;
use crate::models::Peripheral as ModelPeripheral;
use scheduler::Scheduler;

/// A thread safe version of a [Library](../models/struct.Library.html) instance.
///
/// This is a convenience type for sharing a single a Library instance between multiple
/// threads. Due to its use of a Mutex, different peripherals that use the same library will not
/// make function calls from the library in a deterministic order.
pub type TSLibrary = Arc<Mutex<Library>>;

/// A Plugin contains the necessary data to work with a Plugin across the FFI boundary.
///
/// This struct holds a raw pointer to aperipheral that is created by the Peripheral library. In
/// addition it contains the vtable of function pointers defined by the C API and implemented
/// within the Peripheral library.
///
/// The Plugin implements the `Send` trait because after creation the Plugin is moved
/// into the thread that is dedicated to the peripheral that it manages. Once it is moved, it will
/// only ever be owned by this thread by design.
#[derive(Clone, Debug)]
pub struct Plugin {
    object: *mut CPeripheral,
    vtable: VTable,
}

impl Drop for Plugin {
    fn drop(&mut self) {
        (self.vtable.peripheral_free)(self.object);
    }
}

unsafe impl Send for Plugin {}

pub fn init(
    peripheral: &mut ModelPeripheral,
    client: &redis::Client,
    lib: TSLibrary,
) -> std::result::Result<(), PluginInitError> {
    let plugin: *mut CPeripheral =
        unsafe { peripheral_new(lib.clone()).map_err(|e| PluginInitError { side: Box::new(e) })? };
    let vtable: VTable =
        unsafe { peripheral_vtable(lib).map_err(|e| PluginInitError { side: Box::new(e) })? };
    let plugin = Plugin {
        object: plugin,
        vtable: vtable,
    };

    let db = client
        .get_connection()
        .map_err(|e| PluginInitError { side: Box::new(e) })?;

    init::attributes(peripheral, &plugin);

    let mut scheduler = Scheduler::new(plugin, db, peripheral.clone(), SCHEDULER_SLEEP_DURATION);
    init::tasks(&peripheral, &mut scheduler);
    Scheduler::run(scheduler);

    Ok(())
}

unsafe fn peripheral_new(lib: TSLibrary) -> Result<*mut CPeripheral, PeripheralNewError> {
    let lib = lib.lock().map_err(|_| PeripheralNewError {})?;

    let dll = lib.dll().as_ref().ok_or(PeripheralNewError {})?;

    let peripheral_new: Symbol<PeripheralNew> = dll
        .get(b"peripheral_new\0")
        .map_err(|_| PeripheralNewError {})?;

    Ok(peripheral_new())
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
