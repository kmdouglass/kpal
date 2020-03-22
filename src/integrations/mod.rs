//! Integrations provide an entrypoint to KPAL functionality for multiple types of user APIs.
//!
//! The purpose of integrations is to provide a clear interface between the user API and much of
//! the rest of the KPAL crate. This allows developers to more easily add new types of
//! integrations.
//!
//! Examples of possible integrations include
//!
//! - a JSON REST API
//! - gRPC
//! - a C static library
//!
//! The items in the base module are used by specific integrations to interact with the rest of the
//! KPAL crate. Submodules contain implementations of specific integrations.

pub mod rest;

mod errors;

use std::sync::{mpsc::channel, Arc, RwLock};

use crate::{
    constants::REQUEST_TIMEOUT,
    init::{TSLibrary, Transmitters},
    models::{Attribute, Library, Peripheral, PeripheralBuilder, Value},
    plugins::{init as init_plugin, Message},
};

pub use errors::{ErrorReason, IntegrationsError};

/// The Result type that is returned by public functions in the `integrations` module.
type Result<T> = std::result::Result<T, IntegrationsError>;

/// Creates a new peripheral from a peripheral builder and a plugin library.
///
/// The ID of the new peripheral is returned.
///
/// # Arguments
///
/// * `builder` - A PeripheralBuilder instance. This method assumes that none of the builder fields
/// are initialized.
/// * `libs` - The collection of plugin libraries known to KPAL.
/// * `txs` - The collection of transmitters for sending messages into executor threads.
pub fn create_peripheral(
    mut builder: PeripheralBuilder,
    libs: &[TSLibrary],
    txs: Arc<RwLock<Transmitters>>,
) -> Result<usize> {
    let lib = match libs.get(*builder.library_id()) {
        Some(lib) => lib.clone(),
        None => {
            return Err(IntegrationsError::new(
                "Library not found".to_string(),
                ErrorReason::ResourceNotFound,
                None,
            ))
        }
    };

    let id: usize = count_and_incr(txs.clone())?;
    builder = builder.set_id(id);

    init_plugin(builder, lib, txs)?;

    Ok(id)
}

/// Returns the list of plugin libraries currently known to KPAL.
///
/// This method clones the invididual TSLibrary instances into instances of Library that do not
/// contain low-level information about the corresponding shared libraries.
///
/// # Arguments
///
/// * `libs` - The collection of plugin libraries known to KPAL.
pub fn read_libraries(libs: &[TSLibrary]) -> Result<Vec<Library>> {
    let mut result = Vec::new();
    for lib in libs {
        result.push(lib.lock()?.clone());
    }

    Ok(result)
}

/// Returns the Library instance that corresponds to the given ID.
///
/// # Arguments
///
/// * `id` - The ID of the Library to return.
/// * `libs` - The collection of plugin libraries known to KPAL.
pub fn read_library(id: usize, libs: &[TSLibrary]) -> Result<Library> {
    let lib = libs
        .get(id)
        .ok_or_else(|| {
            IntegrationsError::new(
                "Library not found".to_string(),
                ErrorReason::ResourceNotFound,
                None,
            )
        })?
        .lock()?
        .clone();

    Ok(lib)
}

/// Returns the peripheral instance that corresponds to the given ID.
///
/// # Arguments
///
/// * `id` - The ID of the Peripheral to return.
/// * `txs` - The collection of transmitters for sending messages into executor threads.
pub fn read_peripheral(id: usize, txs: Arc<RwLock<Transmitters>>) -> Result<Peripheral> {
    let txs = txs.read()?;
    let ptx = txs
        .get(&id)
        .ok_or_else(|| {
            IntegrationsError::new(
                "Peripheral not found".to_string(),
                ErrorReason::ResourceNotFound,
                None,
            )
        })?
        .lock()?;

    let (tx, rx) = channel();
    let msg = Message::GetPeripheral(tx);
    ptx.send(msg)?;

    rx.recv_timeout(REQUEST_TIMEOUT)?
        .map_err(IntegrationsError::from)
}

/// Returns the current set of peripherals.
///
/// # Arguments
///
/// * `txs` - The collection of transmitters for sending messages into executor threads.
pub fn read_peripherals(txs: Arc<RwLock<Transmitters>>) -> Result<Vec<Peripheral>> {
    let mut msg: Message;
    let mut p: Peripheral;

    let txs = txs.read()?;
    let mut peripherals = Vec::new();
    for (_, mutex) in txs.iter() {
        let ptx = mutex.lock()?;

        let (tx, rx) = channel();
        msg = Message::GetPeripheral(tx);
        ptx.send(msg)?;

        p = rx.recv_timeout(REQUEST_TIMEOUT)??;
        peripherals.push(p);
    }

    Ok(peripherals)
}

/// Returns the peripheral attribute with the given IDs.
///
/// # Arguments
///
/// * `id` - The ID of the Peripheral that owns the Attribute to return.
/// * `attr_id` - The ID of the Attribute to return.
/// * `txs` - The collection of transmitters for sending messages into executor threads.
pub fn read_peripheral_attribute(
    id: usize,
    attr_id: usize,
    txs: Arc<RwLock<Transmitters>>,
) -> Result<Attribute> {
    let txs = txs.read()?;
    let ptx = txs
        .get(&id)
        .ok_or_else(|| {
            IntegrationsError::new(
                "Peripheral not found".to_string(),
                ErrorReason::ResourceNotFound,
                None,
            )
        })?
        .lock()?;

    let (tx, rx) = channel();
    let msg = Message::GetPeripheralAttribute(attr_id, tx);
    ptx.send(msg)?;

    rx.recv_timeout(REQUEST_TIMEOUT)?
        .map_err(IntegrationsError::from)
}

/// Returns all attributes of the peripheral with the given ID.
///
/// # Arguments
///
/// * `id` - The ID of the Peripheral that owns the attributes to return.
/// * `txs` - The collection of transmitters for sending messages into executor threads.
pub fn read_peripheral_attributes(
    id: usize,
    txs: Arc<RwLock<Transmitters>>,
) -> Result<Vec<Attribute>> {
    let txs = txs.read()?;
    let ptx = txs
        .get(&id)
        .ok_or_else(|| {
            IntegrationsError::new(
                "Peripheral not found".to_string(),
                ErrorReason::ResourceNotFound,
                None,
            )
        })?
        .lock()?;

    let (tx, rx) = channel();
    let msg = Message::GetPeripheralAttributes(tx);
    ptx.send(msg)?;

    rx.recv_timeout(REQUEST_TIMEOUT)?
        .map_err(IntegrationsError::from)
}

/// Updates the value of a Peripheral Attribute.
///
/// # Arguments
///
/// * `id` - The ID of the Peripheral that owns the Attribute to return.
/// * `attr_id` - The ID of the Attribute to return.
/// * `value` - The new Value of the Attribute.
/// * `txs` - The collection of transmitters for sending messages into executor threads.
pub fn update_peripheral_attribute(
    id: usize,
    attr_id: usize,
    value: Value,
    txs: Arc<RwLock<Transmitters>>,
) -> Result<Attribute> {
    let txs = txs.read()?;
    let ptx = txs
        .get(&id)
        .ok_or_else(|| {
            IntegrationsError::new(
                "Peripheral not found".to_string(),
                ErrorReason::ResourceNotFound,
                None,
            )
        })?
        .lock()?;

    let (tx, rx) = channel();
    let msg = Message::PatchPeripheralAttribute(attr_id, value, tx);
    ptx.send(msg)?;

    rx.recv_timeout(REQUEST_TIMEOUT)?
        .map_err(IntegrationsError::from)
}

/// Finds and returns the next largest integer to serve as a new peripheral ID.
///
/// This function loops over all the transmitters and finds the largest value for the peripheral
/// ID. It then returns a value that is one greater than this.
///
/// # Arguments
///
/// * `txs` - The collection of transmitters for communicating with peripherals
fn count_and_incr(txs: Arc<RwLock<Transmitters>>) -> Result<usize> {
    let txs = txs.read()?;
    if txs.len() == 0 {
        return Ok(0);
    }

    let mut largest_id: usize = 0;
    for (id, _) in txs.iter() {
        if *id > largest_id {
            largest_id = *id
        }
    }

    Ok(largest_id + 1)
}
