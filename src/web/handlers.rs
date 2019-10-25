//! The set of request handlers for the individual endpoints of the web server.

use std::boxed::Box;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use redis;
use rouille::input::json::json_input;
use rouille::{Request, Response};

use crate::init::transmitters::Transmitters;
use crate::models::database::{Count, Query};
use crate::models::Library;
use crate::models::{Attribute, Peripheral};
use crate::plugins::init as init_plugin;
use crate::plugins::TSLibrary;

/// Handles the GET /api/v0/libraries endpoint.
pub fn get_libraries(db: &redis::Connection) -> Result<Response> {
    let result: Vec<Library> =
        Library::all(&db).map_err(|e| RequestHandlerError { side: Box::new(e) })?;

    Ok(Response::json(&result))
}

/// Handles the GET /api/v0/libraries/{id} endpoint.
pub fn get_library(db: &redis::Connection, id: usize) -> Result<Response> {
    let result: Option<Library> =
        Library::get(&db, id).map_err(|e| RequestHandlerError { side: Box::new(e) })?;

    match result {
        Some(result) => Ok(Response::json(&result)),
        None => Ok(Response::empty_404()),
    }
}
/// Handles the GET /api/v0/peripherals/{id} endpoint.
pub fn get_peripheral(db: &redis::Connection, id: usize) -> Result<Response> {
    let result: Option<Peripheral> =
        Peripheral::get(&db, id).map_err(|e| RequestHandlerError { side: Box::new(e) })?;

    match result {
        Some(result) => Ok(Response::json(&result)),
        None => Ok(Response::empty_404()),
    }
}

/// Handles the GET /api/v0/peripherals endpoint.
pub fn get_peripherals(db: &redis::Connection) -> Result<Response> {
    let result: Vec<Peripheral> =
        Peripheral::all(&db).map_err(|e| RequestHandlerError { side: Box::new(e) })?;

    Ok(Response::json(&result))
}

/// Handles the POST /api/v0/peripherals endpoint.
pub fn post_peripherals(
    request: &Request,
    client: &redis::Client,
    db: &redis::Connection,
    libs: &Vec<TSLibrary>,
    txs: Arc<Transmitters>,
) -> Result<Response> {
    let mut periph: Peripheral =
        json_input(&request).map_err(|e| RequestHandlerError { side: Box::new(e) })?;

    let lib = match libs.get(periph.library_id()) {
        // Bump the reference count on the Arc that wraps this library
        Some(lib) => lib.clone(),
        None => {
            let mut response = Response::text("Library does not exist.\n");
            response.status_code = 400;
            return Ok(response);
        }
    };

    let id: usize =
        Peripheral::count_and_incr(&db).map_err(|e| RequestHandlerError { side: Box::new(e) })?;
    periph.set_id(id);

    init_plugin(&mut periph, client, lib, txs)
        .map_err(|e| RequestHandlerError { side: Box::new(e) })?;

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
    db: &redis::Connection,
    id: usize,
    attr_id: usize,
) -> Result<Response> {
    let peripheral: Peripheral = if let Some(peripheral) =
        Peripheral::get(&db, id).map_err(|e| RequestHandlerError { side: Box::new(e) })?
    {
        peripheral
    } else {
        return Ok(Response::empty_404());
    };

    let result: Option<&Attribute> = peripheral.attributes().get(attr_id);

    match result {
        Some(result) => Ok(Response::json(result)),
        None => Ok(Response::empty_404()),
    }
}

/// Handles the GET /api/v0/peripherals/{id}/attributes endpoint.
pub fn get_peripheral_attributes(db: &redis::Connection, id: usize) -> Result<Response> {
    let result: Option<Peripheral> =
        Peripheral::get(&db, id).map_err(|e| RequestHandlerError { side: Box::new(e) })?;

    match result {
        Some(result) => Ok(Response::json(result.attributes())),
        None => Ok(Response::empty_404()),
    }
}

/// Result type containing a RequestHandlerError for the Err variant.
pub type Result<T> = std::result::Result<T, RequestHandlerError>;

/// An error raised when processing a request.
#[derive(Debug)]
pub struct RequestHandlerError {
    side: Box<dyn Error>,
}

impl Error for RequestHandlerError {
    fn description(&self) -> &str {
        "Failed to handle the HTTP request"
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.side)
    }
}

impl fmt::Display for RequestHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RequestHandlerError {{ Cause: {} }}", &*self.side)
    }
}
