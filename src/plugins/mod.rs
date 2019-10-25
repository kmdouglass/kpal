//! Utilities to create and interact with Plugins.
//!
//! The plugins module is responsible for initializing new plugins and creating their
//! schedulers. Each plugin is assigned a scheduler, and eeach scheduler runs inside its own
//! thread. All communication with a peripheral occurs through tasks that are executed by the
//! scheduler.

mod driver;
mod init;
pub mod messaging;
mod scheduler;

use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};

use libloading::Symbol;

use kpal_plugin::{KpalPluginInit, Plugin};

use crate::constants::SCHEDULER_SLEEP_DURATION;
use crate::init::transmitters::Transmitters;
use crate::models::database::Query;
use crate::models::{Library, Peripheral};
use scheduler::Scheduler;

/// A thread safe version of a [Library](../models/struct.Library.html) instance.
///
/// This is a convenience type for sharing a single a Library instance between multiple
/// threads. Due to its use of a Mutex, different peripherals that use the same library will not
/// make function calls from the library in a deterministic order.
pub type TSLibrary = Arc<Mutex<Library>>;

/// Initializes a new plugin.
///
/// # Arguments
///
/// * `peripheral` - A Peripheral model instance that will be updated with the Plugin's information
/// * `client` - A database client; a clone of this client will be passed to the scheduler
/// * `lib` - A copy of the Library that contains the implementation of the peripheral's Plugin API
/// * `txs` - The set of transmitters currently known to the daemon
pub fn init(
    peripheral: &mut Peripheral,
    client: &redis::Client,
    lib: TSLibrary,
    txs: Arc<Transmitters>,
) -> std::result::Result<(), PluginInitError> {
    let plugin: Plugin = unsafe { kpal_plugin_init(lib.clone())? };

    let db = client
        .get_connection()
        .map_err(|e| PluginInitError { side: Box::new(e) })?;

    init::attributes(peripheral, &plugin);

    let mut scheduler = Scheduler::new(plugin, db, peripheral.clone(), SCHEDULER_SLEEP_DURATION);
    init::tasks(&peripheral, &mut scheduler);

    let tx = Mutex::new(scheduler.tx.clone());
    txs.write()
        .map_err(|_| PluginInitError {
            side: Box::new(TransmittersLockError {}),
        })?
        .insert(peripheral.id(), tx);

    Scheduler::run(scheduler);

    Ok(())
}

/// Requests a new Plugin object from the Library.
///
/// This function is unsafe because it calls a function that is provided by the shared library
/// through the FFI.
///
/// # Arguments
///
/// * `lib` - A copy of the Library that contains the implementation of the peripheral's Plugin API
unsafe fn kpal_plugin_init(lib: TSLibrary) -> Result<Plugin, PluginInitError> {
    let lib = lib.lock().map_err(|_| PluginInitError {
        side: Box::new(LibraryLockError {}),
    })?;

    let dll = lib.dll().as_ref().ok_or(PluginInitError {
        side: Box::new(ReferenceError {}),
    })?;

    let kpal_plugin_init: Symbol<KpalPluginInit> = dll
        .get(b"kpal_plugin_init\0")
        .map_err(|e| PluginInitError { side: Box::new(e) })?;

    Ok(kpal_plugin_init())
}

/// An error caused by any failure in the Plugin initialization routines.
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

/// An error caused by the inability to obtain a lock on a Library instance.
#[derive(Debug)]
pub struct LibraryLockError {}

impl Error for LibraryLockError {
    fn description(&self) -> &str {
        "Could not obtain a lock on the library"
    }
}

impl fmt::Display for LibraryLockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not obtain a lock on the library")
    }
}

/// An error caused by the inability to obtain a reference to a Library instance.
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

/// An error caused by the inability to obtain a lock on the Transmitters.
#[derive(Debug)]
pub struct TransmittersLockError {}

impl Error for TransmittersLockError {
    fn description(&self) -> &str {
        "Could not obtain a lock on the transmitters"
    }
}

impl fmt::Display for TransmittersLockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not obtain a lock on the transmitters")
    }
}
