use rouille::{router, Request, Response};

pub fn routes() -> impl Fn(&Request) -> Response {
    move |request| {
        router!(request,

            // GET /
            (GET) (/) => {
                Response::text("KPAL is under development.")
            },

            // GET /peripherals
            (GET) (/peripherals) => {
                // Returns a list of all peripherals currently registered with the daemon.
                //
                // peripherals are devices or processes that may be controlled by KPAL. At any
                // time, the KPAL daemon has at least one peripheral under its control: itself.
                Response::empty_404()
            },

            // GET /peripherals/{id_peripheral}
            (GET) (/peripherals/{id_peripheral: u32}) => {
                // Returns a single peripheral.
                //
                // The peripheral with an id of 0 is the KPAL daemon itself.
                Response::empty_404()
            },

            // PATCH /peripherals/{id_peripheral}
            (PATCH) (/peripherals/{id_peripheral: u32}) => {
                // Updates the state of a given peripheral.
                Response::empty_404()
            },

            _ => Response::empty_404()
        )
    }
}
