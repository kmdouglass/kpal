use std::process::exit;

use env_logger;
use log;
use rouille::Response;
use structopt::StructOpt;

use kpal::init::{init, Cli};
use kpal::routes::routes;

fn main() {
    env_logger::init();
    let args = Cli::from_args();

    let db = match init(&args) {
        Ok(x) => x,
        Err(e) => {
            log::error!("{}", e);
            exit(1);
        }
    };

    log::info!("Launching the server at {}...", &args.server_addr);
    rouille::start_server(&args.server_addr, move |request| {
        let db = match db.lock() {
            Ok(db) => db,
            Err(_) => return Response::text("Internal server error (500)").with_status_code(500),
        };

        let response = routes(&request, &db);

        response
    });
}
