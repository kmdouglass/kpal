//! The endpoints of the web server.
//!
//! The user API is defined in this module. It is a REST API whose endpoints correspond to the
//! resources of the object model (peripherals, libraries, etc.).

use std::sync::Arc;

use log;
use redis;
use rouille::{router, Request, Response};

use crate::init::transmitters::Transmitters;
use crate::plugins::TSLibrary;
use crate::web::handlers;

/// Directs a HTTP request to the appropriate handler and returns a HTTP response.
///
/// # Arguments
///
/// * `request` - The object containing the information concerning the client's request
/// * `db` - A connection to the database
/// * `libs` The set of libraries that is currently open by the daemon
/// * `transmitters` The set of transmitters for sending messages into each peripheral thread
pub fn routes(
    request: &Request,
    client: &redis::Client,
    db: &redis::Connection,
    libs: &Vec<TSLibrary>,
    txs: Arc<Transmitters>,
) -> Response {
    router!(request,

            (GET) (/) => {
                log::info!("GET /");

                Response::text("Kyle's Peripheral Abstraction Layer (KPAL)")
            },

            (GET) (/api/v0/libraries) => {
                log::info!("GET /api/v0/libraries");
                handlers::get_libraries(&db).unwrap_or_else(log_404)
            },

            (GET) (/api/v0/libraries/{id: usize}) => {
                log::info!("GET /api/v0/libraries/{}", id);
                handlers::get_library(&db, id).unwrap_or_else(log_404)
            },

            (GET) (/api/v0/peripherals) => {
                log::info!("GET /api/v0/peripherals");
                handlers::get_peripherals(&db).unwrap_or_else(log_404)
            },

            (POST) (/api/v0/peripherals) => {
                log::info!("POST /api/v0/peripherals");
                handlers::post_peripherals(&request, &client, &db, &libs, txs).unwrap_or_else(log_404)
            },

            (GET) (/api/v0/peripherals/{id: usize}) => {
                log::info!("GET /api/v0/peripherals/{}", id);
                handlers::get_peripheral(&db, id).unwrap_or_else(log_404)
            },

            (GET) (/api/v0/peripherals/{id: usize}/attributes) => {
                log::info!("GET /api/v0/peripherals/{}/attributes", id);
                handlers::get_peripheral_attributes(&db, id).unwrap_or_else(log_404)
            },

            (GET) (/api/v0/peripherals/{id: usize}/attributes/{attr_id}) => {
                log::info!("GET /api/v0/peripherals/{}/attributes/{}", id, attr_id);
                handlers::get_peripheral_attribute(&db, id, attr_id).unwrap_or_else(log_404)
            },

            _ => Response::empty_404()
    )
}

fn log_404(e: handlers::RequestHandlerError) -> Response {
    log::error!("{}", e);
    Response::empty_404()
}
