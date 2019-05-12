use std::net::SocketAddr;
use std::path::PathBuf;

use dirs::home_dir;
use env_logger;
use lazy_static::lazy_static;
use log;
use structopt::StructOpt;

use kpal::constants::{KPAL_DIR, PERIPHERAL_DIR};
use kpal::peripheral_manager::PeripheralManager;
use kpal::routes::routes;

lazy_static! {
    static ref DEFAULT_PERIPHERAL_DIR: String = {
        let mut default_dir = PathBuf::new();
        default_dir.push(home_dir().expect("Could not determine user's home directory"));
        default_dir.push(KPAL_DIR);
        default_dir.push(PERIPHERAL_DIR);
        default_dir.to_string_lossy().to_string()
    };
}

#[derive(StructOpt)]
struct Cli {
    #[structopt(short = "a", long = "address", default_value = "0.0.0.0:8000")]
    addr: SocketAddr,

    #[structopt(
        short = "p",
        long = "peripheral-dir",
        raw(default_value = "&DEFAULT_PERIPHERAL_DIR"),
        parse(from_os_str)
    )]
    peripheral_dir: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Cli::from_args();

    log::info!(
        "Searching for peripheral library files inside the following directory: {:?}",
        &args.peripheral_dir
    );
    let mut manager = PeripheralManager::new();
    manager
        .init(&args.peripheral_dir)
        .expect("Initialization of the PeripheralManager failed.");

    log::info!("Launching the server at {}...", &args.addr);
    rouille::start_server(&args.addr, move |request| {
        let response = routes(&request, &manager);

        response
    });
}
