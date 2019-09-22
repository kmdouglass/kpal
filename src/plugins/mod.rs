mod driver;
mod init;
mod scheduler;

use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};

use libloading::Symbol;

use kpal_peripheral::{KpalPluginInit, Plugin};

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

pub fn init(
    peripheral: &mut ModelPeripheral,
    client: &redis::Client,
    lib: TSLibrary,
) -> std::result::Result<(), PluginInitError> {
    let plugin: Plugin = unsafe { kpal_plugin_init(lib.clone())? };

    let db = client
        .get_connection()
        .map_err(|e| PluginInitError { side: Box::new(e) })?;

    init::attributes(peripheral, &plugin);

    let mut scheduler = Scheduler::new(plugin, db, peripheral.clone(), SCHEDULER_SLEEP_DURATION);
    init::tasks(&peripheral, &mut scheduler);
    Scheduler::run(scheduler);

    Ok(())
}

unsafe fn kpal_plugin_init(lib: TSLibrary) -> Result<Plugin, PluginInitError> {
    let lib = lib.lock().map_err(|_| PluginInitError {
        side: Box::new(ReferenceError {}),
    })?;

    let dll = lib.dll().as_ref().ok_or(PluginInitError {
        side: Box::new(ReferenceError {}),
    })?;

    let kpal_plugin_init: Symbol<KpalPluginInit> = dll
        .get(b"kpal_plugin_init\0")
        .map_err(|e| PluginInitError { side: Box::new(e) })?;

    Ok(kpal_plugin_init())
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
pub struct LockError {}

impl Error for LockError {
    fn description(&self) -> &str {
        "Could not obtain a lock on the library"
    }
}

impl fmt::Display for LockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not obtain a lock on the library")
    }
}

#[derive(Debug)]
pub struct ReferenceError {}

impl Error for ReferenceError {
    fn description(&self) -> &str {
        "Could not obtain a reference to the library"
    }
}

impl fmt::Display for ReferenceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not obtain a reference to the library")
    }
}
