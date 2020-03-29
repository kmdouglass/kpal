//! Utilities to create and interact with Plugins.
//!
//! The plugins module is responsible for initializing new plugins and creating their
//! executors. Each plugin is assigned an executor, and each executor runs inside its own
//! thread. All communication with a peripheral occurs through tasks that are executed by the
//! executor.

mod errors;
mod executor;
mod messaging;

use std::{
    mem::{discriminant, MaybeUninit},
    sync::{Arc, Mutex, RwLock},
};

use libloading::Symbol;
use log;

use kpal_plugin::error_codes::PLUGIN_OK;
use kpal_plugin::{KpalPluginInit, Plugin};

use crate::{
    init::{TSLibrary, Transmitters},
    models::{Library, Model, PeripheralBuilder},
};

pub use errors::PluginError;
pub use executor::Executor;
pub use messaging::*;

/// Initializes a new plugin.
///
/// # Arguments
///
/// * `peripheral` - A Peripheral model instance that will be updated with the Plugin's information
/// * `lib` - A copy of the Library that contains the implementation of the peripheral's Plugin API
/// * `txs` - The set of transmitters currently known to the daemon
pub fn init(
    builder: PeripheralBuilder,
    lib: TSLibrary,
    txs: Arc<RwLock<Transmitters>>,
) -> std::result::Result<(), PluginError> {
    let plugin: Plugin = {
        let lib = lib.lock()?;
        unsafe { kpal_plugin_new(&lib)? }
    };

    let mut executor = Executor::new(plugin);

    log::debug!("Setting user-specified pre-init attributes");
    let builder = set_attributes(builder, lib)?;

    log::debug!("Synchronizing the plugin with daemon's peripheral data");
    executor.sync(&builder)?;

    log::debug!("Running the plugin's initialization routine");
    executor.init()?;

    log::debug!("Advancing the lifetime phase of the plugin");
    executor.advance()?;

    let peripheral = builder.build()?;

    // Insert the transmitter into the collection of Transmitters only after we have initialized
    // everything successfully. Otherwise, we may insert a channel into the collection which will
    // be immediately closed when this function returns.
    let tx = Mutex::new(executor.tx.clone());
    txs.write()?.insert(peripheral.id(), tx);

    log::debug!("Launching the plugin executor");
    executor.run(peripheral);

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
pub unsafe fn kpal_plugin_new(lib: &Library) -> Result<Plugin, PluginError> {
    let dll = lib.dll().as_ref().ok_or_else(|| {
        PluginError::GetLibraryError(
            "Could not obtain reference to the plugin's shared library".to_string(),
        )
    })?;

    let kpal_plugin_new: Symbol<KpalPluginInit> = dll.get(b"kpal_plugin_new\0")?;

    let mut plugin = MaybeUninit::<Plugin>::uninit();
    let result = kpal_plugin_new(plugin.as_mut_ptr());

    if result != PLUGIN_OK {
        log::error!("Could not create new plugin. Error code: {}", result);
        return Err(PluginError::NewPluginError);
    }

    Ok(plugin.assume_init())
}

/// Set the peripheral attributes, skipping any that are pre-init and have been set by the user.
///
/// # Arguments
///
/// * `builder` - The peripheral builder to which attributes will be added.
/// * `lib` - The plugin library that controls the peripheral
fn set_attributes(
    mut builder: PeripheralBuilder,
    lib: TSLibrary,
) -> Result<PeripheralBuilder, PluginError> {
    let lib = lib.lock()?;
    let lib_attrs = lib.attributes().clone();

    for (id, attr) in lib_attrs {
        // Build all attributes that were provided with initial values from the user.
        let periph_attr = if let Some(mut attr_builder) = builder.attribute_builder(id) {
            attr_builder = attr_builder
                .set_name(attr.name().to_owned())
                .set_pre_init(attr.pre_init());

            Some(attr_builder.build()?)
        } else {
            None
        };

        if let Some(periph_attr) = periph_attr {
            // Verify that user-provided values are valid.
            if discriminant(attr.value()) != discriminant(periph_attr.value()) {
                return Err(PluginError::SetAttributesUserInputError(format!(
                    "Provided attribute variant does not match library's: attribute id {}",
                    id
                )));
            };

            if !periph_attr.pre_init() {
                return Err(PluginError::SetAttributesUserInputError(format!(
                    "Attribute cannot be set before initialization: attribute id {}",
                    id
                )));
            };

            log::debug!(
                "Setting attribute using value provided by the client: {:?}",
                periph_attr
            );
            builder = builder.set_attribute(periph_attr)
        } else {
            log::debug!(
                "Setting attribute using value provided by the library: {:?}",
                attr
            );
            builder = builder.set_attribute(attr);
        };
    }

    Ok(builder)
}
