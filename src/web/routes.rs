//! The endpoints of the web server.
//!
//! The user API is defined in this module. It is a REST API whose endpoints correspond to the
//! resources of the object model (peripherals, libraries, etc.).

use std::sync::{Arc, RwLock};

use log;
use rouille::{router, Request, Response};

use crate::init::TSLibrary;
use crate::init::Transmitters;
use crate::web::handlers;

/// Directs a HTTP request to the appropriate handler and returns a HTTP response.
///
/// # Arguments
///
/// * `request` - The object containing the information concerning the client's request
/// * `libs` The set of libraries that is currently open by the daemon
/// * `transmitters` The set of transmitters for sending messages into each peripheral thread
#[allow(clippy::cognitive_complexity)]
pub fn routes(request: &Request, libs: &[TSLibrary], txs: Arc<RwLock<Transmitters>>) -> Response {
    router!(request,

            (GET) (/) => {
                log::info!("GET /");

                Response::text("Kyle's Peripheral Abstraction Layer (KPAL)")
            },

            (GET) (/api/v0/libraries) => {
                log::info!("GET /api/v0/libraries");
                handlers::get_libraries(libs).unwrap_or_else(log_error)
            },

            (GET) (/api/v0/libraries/{id: usize}) => {
                log::info!("GET /api/v0/libraries/{}", id);
                handlers::get_library(id, libs).unwrap_or_else(log_error)
            },

            (GET) (/api/v0/peripherals) => {
                log::info!("GET /api/v0/peripherals");
                handlers::get_peripherals(txs.clone()).unwrap_or_else(log_error)
            },

            (POST) (/api/v0/peripherals) => {
                log::info!("POST /api/v0/peripherals");
                handlers::post_peripherals(&request, libs, txs.clone()).unwrap_or_else(log_error)
            },

            (GET) (/api/v0/peripherals/{id: usize}) => {
                log::info!("GET /api/v0/peripherals/{}", id);
                handlers::get_peripheral(id, txs.clone()).unwrap_or_else(log_error)
            },

            (GET) (/api/v0/peripherals/{id: usize}/attributes) => {
                log::info!("GET /api/v0/peripherals/{}/attributes", id);
                handlers::get_peripheral_attributes(id, txs.clone()).unwrap_or_else(log_error)
            },

            (GET) (/api/v0/peripherals/{id: usize}/attributes/{attr_id: usize}) => {
                log::info!("GET /api/v0/peripherals/{}/attributes/{}", id, attr_id);
                handlers::get_peripheral_attribute(id, attr_id, txs.clone()).unwrap_or_else(log_error)
            },

            (PATCH) (/api/v0/peripherals/{id: usize}/attributes/{attr_id: usize}) => {
                log::info!("PATCH /api/v0/peripherals/{}/attributes/{}", id, attr_id);
                handlers::patch_peripheral_attribute(&request, id, attr_id, txs).unwrap_or_else(log_error)
            },

            _ => Response::empty_404()
    )
}

fn log_error(e: handlers::HandlerError) -> Response {
    log::error!("{}", e);
    Response::text(e.body).with_status_code(e.http_status_code)
}
