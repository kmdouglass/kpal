use log;
use redis;
use rouille::{router, Request, Response};

use crate::handlers;
use crate::plugins::TSLibrary;

pub fn routes(request: &Request, db: &redis::Connection, libs: &Vec<TSLibrary>) -> Response {
    router!(request,

            // GET /
            (GET) (/) => {
                log::info!("GET /");

                Response::text("Kyle's Peripheral Abstraction Layer (KPAL)")
            },

            // GET /api/v0/libraries
            (GET) (/api/v0/libraries) => {
                log::info!("GET /api/v0/libraries");
                handlers::get_libraries(&db).unwrap_or_else(log_404)
            },

            // GET /api/v0/libraries/{id}
            (GET) (/api/v0/libraries/{id: usize}) => {
                log::info!("GET /api/v0/libraries/{}", id);
                handlers::get_library(&db, id).unwrap_or_else(log_404)
            },

            // GET /api/v0/peripherals
            (GET) (/api/v0/peripherals) => {
                log::info!("GET /api/v0/peripherals");
                handlers::get_peripherals(&db).unwrap_or_else(log_404)
            },

            // POST /api/v0/peripherals
            (POST) (/api/v0/peripherals) => {
                log::info!("POST /api/v0/peripherals");
                handlers::post_peripherals(&request, &db, &libs).unwrap_or_else(log_404)
            },

            // GET /api/v0/peripherals/{id}
            (GET) (/api/v0/peripherals/{id: usize}) => {
                log::info!("GET /api/v0/peripherals/{}", id);
                handlers::get_peripheral(&db, id).unwrap_or_else(log_404)
            },

            // GET /api/v0/peripherals/{id}/attributes
            (GET) (/api/v0/peripherals/{id: usize}/attributes) => {
                log::info!("GET /api/v0/peripherals/{}/attributes", id);
                handlers::get_peripheral_attributes(&db, id).unwrap_or_else(log_404)
            },

            // GET /api/v0/peripherals/{id}/attributes/{attr_id}
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
