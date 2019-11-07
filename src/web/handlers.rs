//! The set of request handlers for the individual endpoints of the web server.

use std::error::Error;
use std::fmt;
use std::sync::mpsc::{channel, RecvTimeoutError, SendError};
use std::sync::{Arc, MutexGuard, PoisonError, RwLock, RwLockReadGuard};

use redis;
use rouille::input::json::{json_input, JsonError};
use rouille::{Request, Response};

use crate::constants::REQUEST_TIMEOUT;
use crate::init::transmitters::Transmitters;
use crate::models::database::{Count, DatabaseError, Query};
use crate::models::Library;
use crate::models::Peripheral;
use crate::plugins::init as init_plugin;
use crate::plugins::messaging::{Message, PluginError, Transmitter};
use crate::plugins::PluginInitError;
use crate::plugins::TSLibrary;

/// Handles the GET /api/v0/libraries endpoint.
pub fn get_libraries(db: &redis::Connection) -> Result<Response> {
    Library::all(&db)
        .map(|l| Response::json(&l))
        .map_err(|e| RequestHandlerError::from(e))
}

/// Handles the GET /api/v0/libraries/{id} endpoint.
pub fn get_library(db: &redis::Connection, id: usize) -> Result<Response> {
    Library::get(&db, id)?
        .map(|l| Response::json(&l))
        .ok_or(ResourceNotFoundError {
            id: id,
            name: String::from(Library::key()),
        })
        .map_err(|e| RequestHandlerError::from(e))
}
/// Handles the GET /api/v0/peripherals/{id} endpoint.
pub fn get_peripheral(db: &redis::Connection, id: usize) -> Result<Response> {
    Peripheral::get(&db, id)?
        .map(|p| Response::json(&p))
        .ok_or(ResourceNotFoundError {
            id: id,
            name: String::from(Peripheral::key()),
        })
        .map_err(|e| RequestHandlerError::from(e))
}

/// Handles the GET /api/v0/peripherals endpoint.
pub fn get_peripherals(db: &redis::Connection) -> Result<Response> {
    Peripheral::all(&db)
        .map(|p| Response::json(&p))
        .map_err(|e| RequestHandlerError::from(e))
}

/// Handles the POST /api/v0/peripherals endpoint.
pub fn post_peripherals(
    request: &Request,
    client: &redis::Client,
    db: &redis::Connection,
    libs: &Vec<TSLibrary>,
    txs: Arc<RwLock<Transmitters>>,
) -> Result<Response> {
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

    let id: usize = Peripheral::count_and_incr(&db)?;
    periph.set_id(id);

    init_plugin(&mut periph, client, lib, txs)?;

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
            id: id,
            name: String::from(Peripheral::key()),
        })?
        .lock()?;

    let (tx, rx) = channel();
    let msg = Message::GetPeripheralAttribute(attr_id, tx);
    ptx.send(msg)?;

    rx.recv_timeout(REQUEST_TIMEOUT)?
        .map(|attr| Response::json(&attr))
        .map_err(|e| RequestHandlerError::from(e))
}

/// Handles the GET /api/v0/peripherals/{id}/attributes endpoint.
pub fn get_peripheral_attributes(id: usize, txs: Arc<RwLock<Transmitters>>) -> Result<Response> {
    let txs = txs.read()?;
    let ptx = txs
        .get(&id)
        .ok_or(ResourceNotFoundError {
            id: id,
            name: String::from(Peripheral::key()),
        })?
        .lock()?;

    let (tx, rx) = channel();
    let msg = Message::GetPeripheralAttributes(tx);
    ptx.send(msg)?;

    rx.recv_timeout(REQUEST_TIMEOUT)?
        .map(|attr| Response::json(&attr))
        .map_err(|e| RequestHandlerError::from(e))
}

/// Result type containing a RequestHandlerError for the Err variant.
pub type Result<T> = std::result::Result<T, RequestHandlerError>;

/// An error raised when a peripheral is not found.
#[derive(Debug)]
pub struct ResourceNotFoundError {
    /// The id of the resource
    id: usize,

    /// The name of the resource collection (e.g. peripherals)
    name: String,
}

impl Error for ResourceNotFoundError {}

impl fmt::Display for ResourceNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Resource not found: {}/{}", self.name, self.id)
    }
}

/// An error raised when processing a request.
#[derive(Debug)]
pub struct RequestHandlerError {
    body: String,
    http_status_code: u16,
}

impl Error for RequestHandlerError {}

impl fmt::Display for RequestHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "RequestHandlerError {{ http_status_code: {}, body: {} }}",
            &self.http_status_code, &self.body
        )
    }
}

impl From<DatabaseError> for RequestHandlerError {
    fn from(error: DatabaseError) -> Self {
        RequestHandlerError {
            body: String::from(format!(
                "Error when communicating with the database: {}",
                error
            )),
            http_status_code: 500,
        }
    }
}

impl From<JsonError> for RequestHandlerError {
    fn from(error: JsonError) -> Self {
        RequestHandlerError {
            body: String::from(format!("Error when serializing to JSON: {}", error)),
            http_status_code: 500,
        }
    }
}

impl From<ResourceNotFoundError> for RequestHandlerError {
    fn from(error: ResourceNotFoundError) -> Self {
        RequestHandlerError {
            body: String::from(format!("Error when accessing a resource: {}", error)),
            http_status_code: 404,
        }
    }
}

impl From<PluginError> for RequestHandlerError {
    fn from(error: PluginError) -> Self {
        RequestHandlerError {
            body: String::from(format!("Error received from plugin: {}", error)),
            http_status_code: error.http_status_code,
        }
    }
}

impl From<PluginInitError> for RequestHandlerError {
    fn from(error: PluginInitError) -> Self {
        RequestHandlerError {
            body: String::from(format!("Error during plugin intitialization: {}", error)),
            http_status_code: 500,
        }
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, Transmitter>>> for RequestHandlerError {
    fn from(error: PoisonError<MutexGuard<Transmitter>>) -> Self {
        RequestHandlerError {
            body: String::from(format!("Peripheral thread is poisoned: {}", error)),
            http_status_code: 500,
        }
    }
}

impl<'a> From<PoisonError<RwLockReadGuard<'a, Transmitters>>> for RequestHandlerError {
    fn from(error: PoisonError<RwLockReadGuard<Transmitters>>) -> Self {
        RequestHandlerError {
            body: String::from(format!("Transmitters thread is poisoned: {}", error)),
            http_status_code: 500,
        }
    }
}

impl From<RecvTimeoutError> for RequestHandlerError {
    fn from(error: RecvTimeoutError) -> Self {
        RequestHandlerError {
            body: String::from(format!("Timeout while waiting on peripheral: {}", error)),
            http_status_code: 500,
        }
    }
}

impl From<SendError<Message>> for RequestHandlerError {
    fn from(error: SendError<Message>) -> Self {
        RequestHandlerError {
            body: String::from(format!("Unable to send message to peripheral: {}", error)),
            http_status_code: 500,
        }
    }
}
