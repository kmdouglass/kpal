//! Utilities to create and interact with Plugins.
//!
//! The plugins module is responsible for initializing new plugins and creating their
//! executors. Each plugin is assigned an executor, and each executor runs inside its own
//! thread. All communication with a peripheral occurs through tasks that are executed by the
//! executor.

mod errors;
mod executor;
pub mod messaging;

use std::{
    mem::MaybeUninit,
    sync::{Arc, Mutex, RwLock},
};

use libloading::Symbol;

use kpal_plugin::constants::PLUGIN_OK;
use kpal_plugin::{KpalPluginInit, Plugin};

use crate::init::libraries::TSLibrary;
use crate::init::transmitters::Transmitters;
use crate::models::{Model, Peripheral};
use executor::Executor;

pub use errors::*;

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
) -> std::result::Result<(), PluginError> {
    let plugin: Plugin = unsafe { kpal_plugin_init(lib)? };

    let mut executor = Executor::new(plugin, peripheral.clone());
    executor.init_attributes();

    let tx = Mutex::new(executor.tx.clone());
    txs.write()?.insert(peripheral.id(), tx);

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
unsafe fn kpal_plugin_init(lib: TSLibrary) -> Result<Plugin, PluginError> {
    let lib = lib.lock()?;

    let dll = lib.dll().as_ref().ok_or(PluginError {
        body: "Could not obtain reference to the plugin's shared library".to_string(),
        http_status_code: 500,
    })?;

    let kpal_plugin_init: Symbol<KpalPluginInit> = dll.get(b"kpal_plugin_init\0")?;

    let mut plugin = MaybeUninit::<Plugin>::uninit();
    let result = kpal_plugin_init(plugin.as_mut_ptr());

    if result != PLUGIN_OK {
        log::error!("Plugin initialization failed: {}", result);
        return Err(PluginError {
            body: "Could not initialize plugin".to_string(),
            http_status_code: 500,
        });
    }

    Ok(plugin.assume_init())
}
