use rouille::{router, Request, Response};
use std::net::SocketAddr;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(short = "a", long = "address", default_value = "0.0.0.0:8000")]
    addr: SocketAddr,
}

fn main() {
    let args = Cli::from_args();

    rouille::start_server(&args.addr, move |request| {
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

            // GET /peripherals/streams
            (GET) (/peripherals/streams) => {
                // Returns a list of all streams of all peripherals currently managed by KPAL.
                //
                // streams are two-way communication channels between the client and the daemon.
                Response::empty_404()
            },

            // GET /peripherals/{id_peripheral}/streams
            (GET) (/peripherals/{id_peripheral: u32}/streams) => {
                // Returns all the streams of the given peripheral.
                Response::empty_404()
            },

            // GET /peripherals/{id_peripheral}/streams/{id_stream}
            (GET) (/peripherals/{id_peripheral: u32}/streams/{id_stream: u32}) => {
                // Returns the given stream of the given peripheral.
                Response::empty_404()
            },

            // PATCH /peripherals/{id_peripheral}/streams/{id_stream}
            (PATCH) (/peripherals/{id_peripheral: u32}/streams/{id_stream: u32}) => {
                // Updates the given stream of the given peripheral.
                Response::empty_404()
            },

            _ => Response::empty_404()
        )
    });
}
