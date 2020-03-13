//! Functions for processing requests to the REST integration.
mod errors;

use std::{
    convert::{TryFrom, TryInto},
    sync::{Arc, RwLock},
};

use rouille::input::json::json_input;
use rouille::{Request, Response};

use crate::{
    init::{TSLibrary, Transmitters},
    integrations::{
        create_peripheral, read_libraries, read_library, read_peripheral,
        read_peripheral_attribute, read_peripheral_attributes, read_peripherals,
        update_peripheral_attribute,
    },
    models::{PeripheralBuilder, Value},
};

use super::schemas::{
    AttributeRead, LibraryRead, PeripheralCreate, PeripheralCreateResponse, PeripheralRead,
    SchemaError, ValueReadUpdate,
};

pub use errors::RestHandlerError;

/// The Result type returned by the REST handlers.
type Result<T> = std::result::Result<T, RestHandlerError>;

/// Handles the GET /api/v0/libraries endpoint.
///
/// # Arguments
///
/// * `libs` - The collection of plugin libraries known to KPAL.
pub fn get_libraries(libs: &[TSLibrary]) -> Result<Response> {
    let libs = read_libraries(libs)?;

    let response: Vec<LibraryRead> =
        libs.into_iter()
            .map(|lib| lib.try_into())
            .collect::<std::result::Result<Vec<LibraryRead>, SchemaError>>()?;

    Ok(Response::json(&response))
}

/// Handles the GET /api/v0/libraries/{id} endpoint.
///
/// # Arguments
///
/// * `id` - The ID of the Library to return.
/// * `libs` - The collection of plugin libraries known to KPAL.
pub fn get_library(id: usize, libs: &[TSLibrary]) -> Result<Response> {
    let lib = read_library(id, libs)?;

    let response = LibraryRead::try_from(lib)?;

    Ok(Response::json(&response))
}

/// Handles the GET /api/v0/peripherals/{id} endpoint.
///
/// # Arguments
///
/// * `id` - The ID of the Peripheral to return.
/// * `txs` - The collection of transmitters for sending messages into executor threads.
pub fn get_peripheral(id: usize, txs: Arc<RwLock<Transmitters>>) -> Result<Response> {
    let periph = read_peripheral(id, txs)?;

    let response = PeripheralRead::from(periph);

    Ok(Response::json(&response))
}

/// Handles the GET /api/v0/peripherals/{id}/attributes/{attr_id} endpoint.
///
/// # Arguments
///
/// * `id` - The ID of the Peripheral that owns the Attribute to return.
/// * `attr_id` - The ID of the Attribute to return.
/// * `txs` - The collection of transmitters for sending messages into executor threads.
pub fn get_peripheral_attribute(
    id: usize,
    attr_id: usize,
    txs: Arc<RwLock<Transmitters>>,
) -> Result<Response> {
    let attr = read_peripheral_attribute(id, attr_id, txs)?;

    let response = AttributeRead::try_from(attr)?;

    Ok(Response::json(&response))
}

/// Handles the GET /api/v0/peripherals/{id}/attributes endpoint.
///
/// # Arguments
///
/// * `id` - The ID of the Peripheral that owns the attributes to return.
/// * `txs` - The collection of transmitters for sending messages into executor threads.
pub fn get_peripheral_attributes(id: usize, txs: Arc<RwLock<Transmitters>>) -> Result<Response> {
    let attrs = read_peripheral_attributes(id, txs)?;

    let response: Vec<AttributeRead> = attrs
        .into_iter()
        .map(|attr| attr.try_into())
        .collect::<std::result::Result<Vec<AttributeRead>, SchemaError>>(
    )?;

    Ok(Response::json(&response))
}

/// Handles the GET /api/v0/peripherals endpoint.
///
/// # Arguments
///
/// * `txs` - The collection of transmitters for sending messages into executor threads.
pub fn get_peripherals(txs: Arc<RwLock<Transmitters>>) -> Result<Response> {
    let periphs = read_peripherals(txs)?;

    let response: Vec<PeripheralRead> = periphs.into_iter().map(|periph| periph.into()).collect();

    Ok(Response::json(&response))
}

/// Handles the PATCH /api/v0/peripherals/{id}/attributes/{attr_id} endpoint.
///
/// # Arguments
///
/// * `request` - The request object that contains the user-provided request data.
/// * `id` - The ID of the Peripheral that owns the Attribute to return.
/// * `attr_id` - The ID of the Attribute to return.
/// * `txs` - The collection of transmitters for sending messages into executor threads.
pub fn patch_peripheral_attribute(
    request: &Request,
    id: usize,
    attr_id: usize,
    txs: Arc<RwLock<Transmitters>>,
) -> Result<Response> {
    let data: ValueReadUpdate = json_input(&request)?;
    let value = Value::try_from(data)?;

    let attr = update_peripheral_attribute(id, attr_id, value, txs)?;

    let response = AttributeRead::try_from(attr)?;

    Ok(Response::json(&response))
}

/// Handles the POST /api/v0/peripherals endpoint.
///
/// # Arguments
///
/// * `request` - The request object that contains the user-provided request data.
/// * `libs` - The collection of plugin libraries known to KPAL.
/// * `txs` - The collection of transmitters for sending messages into executor threads.
pub fn post_peripherals(
    request: &Request,
    libs: &[TSLibrary],
    txs: Arc<RwLock<Transmitters>>,
) -> Result<Response> {
    let data: PeripheralCreate = json_input(&request)?;
    let builder = PeripheralBuilder::try_from(data)?;

    let id = create_peripheral(builder, libs, txs)?;

    let location = format!("/api/v0/peripherals/{}", id);
    let mut response = Response::json(&PeripheralCreateResponse {
        message: format!("Peripheral successfully created. Location: {}", location),
    });
    response.status_code = 201;
    response.headers.push(("Location".into(), location.into()));

    Ok(response)
}
