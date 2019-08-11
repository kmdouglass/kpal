use log;
use redis;
use rouille::{router, Request, Response};

use crate::handlers;
use crate::models::Library;

pub fn routes(request: &Request, db: &redis::Connection, libs: &Vec<Library>) -> Response {
    router!(request,

            // GET /
            (GET) (/) => {
                log::info!("GET /");

                Response::text("Kyle's Peripheral Abstraction Layer (KPAL)")
            },

            // GET /api/v0/libraries
            (GET) (/api/v0/libraries) => {
                log::info!("GET /api/v0/libraries");
                handlers::get_libraries(&db).unwrap_or_else(|e| {
                    log::error!("{}", e);
                    Response::empty_404()
                })
            },

            // GET /api/v0/libraries/{id}
            (GET) (/api/v0/libraries/{id: usize}) => {
                log::info!("GET /api/v0/libraries/{}", id);
                handlers::get_library(&db, id).unwrap_or_else(|e| {
                    log::error!("{}", e);
                    Response::empty_404()
                })
            },

            // GET /api/v0/peripherals
            (GET) (/api/v0/peripherals) => {
                log::info!("GET /api/v0/peripherals");
                handlers::get_peripherals(&db).unwrap_or_else(|e| {
                    log::error!("{}", e);
                    Response::empty_404()
                })
            },

            // POST /api/v0/peripherals
            (POST) (/api/v0/peripherals) => {
                log::info!("POST /api/v0/peripherals");
                handlers::post_peripherals(&request, &db, &libs).unwrap_or_else(|e| {
                    log::error!("{}", e);
                    Response::empty_400()
                })
            },

            // GET /peripherals/{id}
            (GET) (/api/v0/peripherals/{id: usize}) => {
                log::info!("GET /api/v0/peripherals/{}", id);
                handlers::get_peripheral(&db, id).unwrap_or_else(|e| {
                    log::error!("{}", e);
                    Response::empty_404()
                })
            },

            _ => Response::empty_404()
    )
}
