use std::process::exit;

use env_logger;
use log;
use structopt::StructOpt;

use kpal::init::{init, Cli};
use kpal::routes::routes;

fn main() {
    env_logger::init();
    let args = Cli::from_args();

    // TODO Feed the return value to the router
    let _ = match init(&args) {
        Ok(x) => x,
        Err(e) => {
            log::error!("{}", e);
            exit(1);
        }
    };

    log::info!("Launching the server at {}...", &args.server_addr);
    rouille::start_server(&args.server_addr, move |request| {
        let response = routes(&request);

        response
    });
}
