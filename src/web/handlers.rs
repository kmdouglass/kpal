//! The set of request handlers for the individual endpoints of the web server.

use std::sync::mpsc::channel;
use std::sync::{Arc, RwLock};

use rouille::input::json::json_input;
use rouille::{Request, Response};

use crate::constants::REQUEST_TIMEOUT;
use crate::init::libraries::TSLibrary;
use crate::init::transmitters::Transmitters;
use crate::models::{Library, Model, Peripheral, Value};
use crate::plugins::{init as init_plugin, messaging::Message};

pub use super::errors::RequestHandlerError;
use super::errors::*;

/// Handles the GET /api/v0/libraries endpoint.
pub fn get_libraries(libs: &[TSLibrary]) -> Result<Response> {
    let mut result = Vec::new();
    for lib in libs {
        result.push(lib.lock()?.clone());
    }

    Ok(Response::json(&result))
}

/// Handles the GET /api/v0/libraries/{id} endpoint.
pub fn get_library(id: usize, libs: &[TSLibrary]) -> Result<Response> {
    let lib = libs
        .get(id)
        .ok_or(ResourceNotFoundError {
            id,
            name: String::from(Library::key()),
        })?
        .lock()?;

    Ok(Response::json(&*lib))
}
/// Handles the GET /api/v0/peripherals/{id} endpoint.
pub fn get_peripheral(id: usize, txs: Arc<RwLock<Transmitters>>) -> Result<Response> {
    let txs = txs.read()?;
    let ptx = txs
        .get(&id)
        .ok_or(ResourceNotFoundError {
            id,
            name: String::from(Peripheral::key()),
        })?
        .lock()?;

    let (tx, rx) = channel();
    let msg = Message::GetPeripheral(tx);
    ptx.send(msg)?;

    rx.recv_timeout(REQUEST_TIMEOUT)?
        .map(|attr| Response::json(&attr))
        .map_err(RequestHandlerError::from)
}

/// Handles the GET /api/v0/peripherals endpoint.
pub fn get_peripherals(txs: Arc<RwLock<Transmitters>>) -> Result<Response> {
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

    Ok(Response::json(&peripherals))
}

/// Handles the POST /api/v0/peripherals endpoint.
pub fn post_peripherals(
    request: &Request,
    libs: &[TSLibrary],
    txs: Arc<RwLock<Transmitters>>,
) -> Result<Response> {
    // NOTE Attributes that are required for initialization will come in with the request here.
    let mut periph: Peripheral = json_input(&request)?;

    let lib = match libs.get(periph.library_id()) {
        // Bump the reference count on the Arc that wraps this library
        Some(lib) => lib.clone(),
        None => {
            let mut response = Response::text("Library does not exist.\n");
            response.status_code = 400;
            return Ok(response);
        }
    };

    let id: usize = count_and_incr(txs.clone())?;
    periph.set_id(id);

    init_plugin(&mut periph, lib, txs)?;

    let mut response = Response::text("The peripheral has been created.\n");
    response.status_code = 201;
    response.headers.push((
        "Location".into(),
        format!("/api/v0/peripherals/{}", &periph.id()).into(),
    ));

    Ok(response)
}

/// Handles the GET /api/v0/peripherals/{id}/attributes/{attr_id} endpoint.
pub fn get_peripheral_attribute(
    id: usize,
    attr_id: usize,
    txs: Arc<RwLock<Transmitters>>,
) -> Result<Response> {
    let txs = txs.read()?;
    let ptx = txs
        .get(&id)
        .ok_or(ResourceNotFoundError {
            id,
            name: String::from(Peripheral::key()),
        })?
        .lock()?;

    let (tx, rx) = channel();
    let msg = Message::GetPeripheralAttribute(attr_id, tx);
    ptx.send(msg)?;

    rx.recv_timeout(REQUEST_TIMEOUT)?
        .map(|attr| Response::json(&attr))
        .map_err(RequestHandlerError::from)
}

/// Handles the PATCH /api/v0/peripherals/{id}/attributes/{attr_id} endpoint.
pub fn patch_peripheral_attribute(
    request: &Request,
    id: usize,
    attr_id: usize,
    txs: Arc<RwLock<Transmitters>>,
) -> Result<Response> {
    let value: Value = json_input(&request)?;

    let txs = txs.read()?;
    let ptx = txs
        .get(&id)
        .ok_or(ResourceNotFoundError {
            id,
            name: String::from(Peripheral::key()),
        })?
        .lock()?;

    let (tx, rx) = channel();
    let msg = Message::PatchPeripheralAttribute(attr_id, value, tx);
    ptx.send(msg)?;

    rx.recv_timeout(REQUEST_TIMEOUT)?
        .map(|attr| Response::json(&attr))
        .map_err(RequestHandlerError::from)
}

/// Handles the GET /api/v0/peripherals/{id}/attributes endpoint.
pub fn get_peripheral_attributes(id: usize, txs: Arc<RwLock<Transmitters>>) -> Result<Response> {
    let txs = txs.read()?;
    let ptx = txs
        .get(&id)
        .ok_or(ResourceNotFoundError {
            id,
            name: String::from(Peripheral::key()),
        })?
        .lock()?;

    let (tx, rx) = channel();
    let msg = Message::GetPeripheralAttributes(tx);
    ptx.send(msg)?;

    rx.recv_timeout(REQUEST_TIMEOUT)?
        .map(|attr| Response::json(&attr))
        .map_err(RequestHandlerError::from)
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
