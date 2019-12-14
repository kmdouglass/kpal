//! Utilities to create and interact with Plugins.
//!
//! The plugins module is responsible for initializing new plugins and creating their
//! executors. Each plugin is assigned an executor, and each executor runs inside its own
//! thread. All communication with a peripheral occurs through tasks that are executed by the
//! executor.

mod driver;
mod executor;
mod init;
pub mod messaging;

use std::error::Error;
use std::fmt;
use std::mem::MaybeUninit;
use std::sync::{Arc, Mutex, RwLock};

use libloading::Symbol;

use kpal_plugin::constants::PLUGIN_OK;
use kpal_plugin::{KpalPluginInit, Plugin};

use crate::init::libraries::TSLibrary;
use crate::init::transmitters::Transmitters;
use crate::models::{Model, Peripheral};
use executor::Executor;

/// Initializes a new plugin.
///
/// # Arguments
///
/// * `peripheral` - A Peripheral model instance that will be updated with the Plugin's information
/// * `lib` - A copy of the Library that contains the implementation of the peripheral's Plugin API
/// * `txs` - The set of transmitters currently known to the daemon
pub fn init(
    peripheral: &mut Peripheral,
    lib: TSLibrary,
    txs: Arc<RwLock<Transmitters>>,
) -> std::result::Result<(), PluginInitError> {
    let plugin: Plugin = unsafe { kpal_plugin_init(lib.clone())? };

    init::attributes(peripheral, &plugin);

    let executor = Executor::new(plugin, peripheral.clone(), lib.clone());

    let tx = Mutex::new(executor.tx.clone());
    txs.write()
        .map_err(|_| PluginInitError {
            side: Box::new(TransmittersLockError {}),
        })?
        .insert(peripheral.id(), tx);

    executor.run();

    Ok(())
}

/// Requests a new Plugin object from the Library.
///
/// # Safety
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

    let mut plugin = MaybeUninit::<Plugin>::uninit();
    let result = kpal_plugin_init(plugin.as_mut_ptr());

    if result != PLUGIN_OK {
        log::error!("Plugin initialization failed: {}", result);
        return Err(PluginInitError {
            side: Box::new(FFIError {}),
        });
    }

    Ok(plugin.assume_init())
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

/// An error caused by a failure to initialize the Plugin on the other side of the FFI layer.
#[derive(Debug)]
pub struct FFIError {}

impl Error for FFIError {
    fn description(&self) -> &str {
        "Failed to intialize the plugin on the other side of the FFI"
    }
}

impl fmt::Display for FFIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Failed to intialize the plugin on the other side of the FFI"
        )
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
