//! Routines for initializing the daemon.
pub mod database;
pub mod library;

use std::boxed::Box;
use std::error::Error;
use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Mutex;

use dirs::home_dir;
use lazy_static::lazy_static;
use redis;
use structopt::StructOpt;
use url::Url;

use crate::constants::{KPAL_DIR, LIBRARY_DIR};
use crate::plugins::TSLibrary;

lazy_static! {
    static ref DEFAULT_LIBRARY_DIR: String = {
        let mut default_dir = PathBuf::new();
        default_dir.push(home_dir().expect("Could not determine user's home directory"));
        default_dir.push(KPAL_DIR);
        default_dir.push(LIBRARY_DIR);
        default_dir.to_string_lossy().to_string()
    };
}

/// The set of command line arguments for the daemon.
#[derive(StructOpt)]
#[structopt(
    name = "kpald",
    about = "A general-purpose control system for physical computing"
)]
pub struct Cli {
    #[structopt(short = "s", long = "server-address", default_value = "0.0.0.0:8000")]
    pub server_addr: SocketAddr,

    #[structopt(
        short = "d",
        long = "database-address",
        default_value = "redis://127.0.0.1:6379"
    )]
    pub db_addr: Url,

    #[structopt(
        short = "l",
        long = "library-dir",
        raw(default_value = "&DEFAULT_LIBRARY_DIR"),
        parse(from_os_str)
    )]
    pub library_dir: PathBuf,
}

/// Initializes the daemon.
///
/// This method returns the data structures that are required by the daemon to operate, including a
/// database client, a connection (for use by the route handlers), and a vector of thread-safe
/// libraries that have been loaded into memory.
///
/// # Arguments
///
/// * `args` - The command line arguments that were passed to the daemon at startup.
pub fn init(args: &Cli) -> Result<(redis::Client, Mutex<redis::Connection>, Vec<TSLibrary>)> {
    let libs = library::init(&args.library_dir).map_err(|e| InitError { side: Box::new(e) })?;
    let (client, db) =
        database::init(&args.db_addr, &libs).map_err(|e| InitError { side: Box::new(e) })?;

    Ok((client, db, libs))
}

/// A Result that is returned by this module.
pub type Result<T> = std::result::Result<T, InitError>;

/// Raised  when an error occurs during the daemon's initialization.
#[derive(Debug)]
pub struct InitError {
    side: Box<dyn Error>,
}

impl Error for InitError {
    fn description(&self) -> &str {
        "Failed to initialize the daemon"
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.side)
    }
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InitError {{ Cause: {} }}", &*self.side)
    }
}
