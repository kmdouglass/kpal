use log;
use redis;
use rouille::{router, Request, Response};

use crate::handlers;

pub fn routes(request: &Request, db: &redis::Connection) -> Response {
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
                handlers::get_libraries_id(&db, id).unwrap_or_else(|e| {
                    log::error!("{}", e);
                    Response::empty_404()
                })
            },
    /*
            // GET /peripherals
            (GET) (/peripherals) => {
                // Returns a list of all peripherals currently registered with the daemon.
                //
                // peripherals are devices or processes that may be controlled by KPAL.
                Response::empty_404()
            },

            // GET /peripherals/{id}
            (GET) (/peripherals/{id: usize}) => {
                // Returns a single peripheral.
                Response::empty_404()
            },

            // PATCH /peripherals/{id}
            (PATCH) (/peripherals/{id: usize}) => {
                // Updates the state of a given peripheral.
                Response::empty_404()
            },
    */
            _ => Response::empty_404()
        )
}
