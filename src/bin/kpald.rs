use std::process::exit;
use std::sync::Arc;

use env_logger;
use log;
use structopt::StructOpt;

use kpal::init::{init, Cli, Init};
use kpal::web::routes;

fn main() {
    env_logger::init();
    let args = Cli::from_args();

    let Init {
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
        let transmitters = transmitters.clone();

        routes(&request, &libraries, transmitters)
    });
}
