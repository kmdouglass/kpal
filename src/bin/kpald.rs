use std::net::SocketAddr;

use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(short = "a", long = "address", default_value = "0.0.0.0:8000")]
    addr: SocketAddr,
}

fn main() {
    let args = Cli::from_args();

    // Launches the REST server
    rouille::start_server(&args.addr, kpal::server::api());
}
