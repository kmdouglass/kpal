use std::process::exit;
use std::sync::Arc;

use env_logger;
use log;
use rouille::Response;
use structopt::StructOpt;

use kpal::init::{init, Cli, Init};
use kpal::web::routes::routes;

fn main() {
    env_logger::init();
    let args = Cli::from_args();

    let Init {
        client,
        db,
        libraries,
        transmitters,
    } = match init(&args) {
        Ok(init) => init,
        Err(e) => {
            log::error!("{}", e);
            exit(1);
        }
    };

    let transmitters = Arc::new(transmitters);

    log::info!("Launching the server at {}...", &args.server_addr);
    rouille::start_server(&args.server_addr, move |request| {
        let db = match db.lock() {
            Ok(db) => db,
            Err(_) => return Response::text("Internal server error (500)").with_status_code(500),
        };
        let transmitters = transmitters.clone();

        let response = routes(&request, &client, &db, &libraries, transmitters);

        response
    });
}
