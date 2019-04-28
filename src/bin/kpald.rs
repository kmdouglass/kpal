use std::net::SocketAddr;
use std::path::PathBuf;

use dirs::home_dir;
use env_logger;
use lazy_static::lazy_static;
use log;
use structopt::StructOpt;

use kpal::constants::{KPAL_DIR, PLUGIN_DIR};
use kpal::routes::routes;
use kpal::utils::find_plugins;

lazy_static! {
    static ref DEFAULT_PLUGIN_DIR: String = {
        let mut default_dir = PathBuf::new();
        default_dir.push(home_dir().expect("Could not determine user's home directory"));
        default_dir.push(KPAL_DIR);
        default_dir.push(PLUGIN_DIR);
        default_dir.to_string_lossy().to_string()
    };
}

#[derive(StructOpt)]
struct Cli {
    #[structopt(short = "a", long = "address", default_value = "0.0.0.0:8000")]
    addr: SocketAddr,

    #[structopt(
        short = "p",
        long = "plugin-dir",
        raw(default_value = "&DEFAULT_PLUGIN_DIR"),
        parse(from_os_str)
    )]
    plugin_dir: PathBuf,
}

fn main() {
    env_logger::init();
    let args = Cli::from_args();

    log::info!(
        "Searching for plugins inside the following directory: {:?}",
        &args.plugin_dir
    );
    let plugin_libs: Vec<PathBuf> = match find_plugins(&args.plugin_dir).expect("error") {
        Some(libs) => libs,
        None => {
            log::info!("No plugin library files found in {:?}", &args.plugin_dir);
            Vec::new()
        }
    };

    // Launches the REST server
    rouille::start_server(&args.addr, routes());
}
