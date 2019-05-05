use std::collections::HashMap;

use rouille::{router, Request, Response};

use crate::peripherals::Peripheral;

pub fn routes(request: &Request, peripherals: &HashMap<usize, Peripheral>) -> Response {
    router!(request,

        // GET /
        (GET) (/) => {
            Response::text("KPAL is under development.")
        },

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

        _ => Response::empty_404()
    )
}
