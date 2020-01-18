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
    mem::{discriminant, MaybeUninit},
    sync::{Arc, Mutex, RwLock},
};

use libloading::Symbol;
use log;

use kpal_plugin::error_codes::PLUGIN_OK;
use kpal_plugin::{KpalPluginInit, Plugin};

use crate::init::libraries::TSLibrary;
use crate::init::transmitters::Transmitters;
use crate::models::{Library, Model, Peripheral};

pub use errors::*;
pub use executor::Executor;

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
    let plugin: Plugin = {
        let lib = lib.lock()?;
        unsafe { kpal_plugin_new(&lib)? }
    };

    let mut executor = Executor::new(plugin);

    // Set any pre-init attributes
    merge_attributes(peripheral, lib)?;
    peripheral.set_attribute_links();

    log::debug!("Synchronizing the plugin with daemon's peripheral data");
    executor.sync(peripheral)?;

    log::debug!("Running the plugin's initialization routine");
    executor.init()?;

    log::debug!("Advancing the lifetime phase of the plugin");
    executor.advance()?;

    // Insert the transmitter into the collection of Transmitters only after we have initialized
    // everything successfully. Otherwise, we may insert a channel into the collection which will
    // be immediately closed when this function returns.
    let tx = Mutex::new(executor.tx.clone());
    txs.write()?.insert(peripheral.id(), tx);

    log::debug!("Launching the plugin executor");
    executor.run(peripheral.clone());

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
    let dll = lib.dll().as_ref().ok_or(PluginError {
        body: "Could not obtain reference to the plugin's shared library".to_string(),
        http_status_code: 500,
    })?;

    let kpal_plugin_new: Symbol<KpalPluginInit> = dll.get(b"kpal_plugin_new\0")?;

    let mut plugin = MaybeUninit::<Plugin>::uninit();
    let result = kpal_plugin_new(plugin.as_mut_ptr());

    if result != PLUGIN_OK {
        log::error!("Plugin initialization failed: {}", result);
        return Err(PluginError {
            body: "Could not initialize plugin".to_string(),
            http_status_code: 500,
        });
    }

    Ok(plugin.assume_init())
}

/// Merge the attributes of the library model into those of the Peripheral.
///
/// This function enables users to set attribute values before a plugin is initialized. It takes
/// the attribute values that are input from the user, which is stored in the Peripheral instance,
/// and merges it into the list of Attributes that the library provides. Then, it replaces the list
/// of attributes inside the Peripheral instance with this updated list.
///
/// This method must be updated anytime a new Attribute variant is added.
///
/// TODO Finish docstring
fn merge_attributes(periph: &mut Peripheral, lib: TSLibrary) -> Result<(), MergeAttributesError> {
    use crate::models::Attribute::*;

    let lib = lib.lock()?;
    let mut attrs = lib.attributes().clone();

    for periph_attr in periph.attributes() {
        let id = periph_attr.id();

        let attr = attrs.get_mut(id).ok_or_else(|| {
            MergeAttributesError::DoesNotExist(format!("Attribute does not exist: {}", id))
        })?;

        if discriminant(attr) != discriminant(periph_attr) {
            return Err(MergeAttributesError::VariantMismatch(format!(
                "Provided variant does not match plugin attribute: {}",
                id
            )));
        };

        let err = MergeAttributesError::IsNotPreInit(
            "Attribute cannot be set before initialization".to_string(),
        );
        #[rustfmt::skip]
        match (attr, periph_attr) {
            (Int { pre_init, value: old_value, .. }, Int { value: new_value, .. }) => {
                if !(*pre_init) { return Err(err); }
                *old_value = *new_value
            }
            (Double { pre_init, value: old_value, .. }, Double { value: new_value, .. }) => {
                if !(*pre_init) { return Err(err); }
                *old_value = *new_value
            }
            (String { pre_init, value: old_value, .. }, String { value: new_value, .. }) => {
                if !(*pre_init) { return Err(err); }
                *old_value = new_value.clone()
            }
            (_, _) => {
                return Err(MergeAttributesError::UnknownVariant(
                    "The daemon does not know how to merge this variant".to_string(),
                ))
            }
        };
    }

    periph.set_attributes(attrs);
    Ok(())
}
