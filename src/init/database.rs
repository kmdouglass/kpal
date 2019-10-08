//! Methods for initializing the database.
use std::boxed::Box;
use std::error::Error;
use std::fmt;
use std::sync::Mutex;

use log;
use redis;
use serde_json;
use url::Url;

use crate::constants::DATABASE_INDEX;
use crate::models::database::{init as model_init, Count, Query};
use crate::models::Library;
use crate::plugins::TSLibrary;

// TODO Provide a connection pool rather than a single mutex to the database connection
/// Returns an initialized connection to the database.
///
/// # Arguments
///
/// * `db_addr` - The socket address of the database
/// * `libs` - A vector of libraries that have been loaded into memory
pub fn init(
    db_addr: &Url,
    libs: &Vec<TSLibrary>,
) -> Result<(redis::Client, Mutex<redis::Connection>), DatabaseInitError> {
    let mut db_addr = db_addr.clone();
    db_addr.set_path(DATABASE_INDEX);
    log::info!("Initializing the database connection to {}", db_addr);

    let client =
        redis::Client::open(db_addr).map_err(|e| DatabaseInitError { side: Box::new(e) })?;

    let connection = client
        .get_connection()
        .map_err(|e| DatabaseInitError { side: Box::new(e) })?;

    log::debug!("Flushing database number {}", DATABASE_INDEX);
    redis::cmd("FLUSHALL")
        .query(&connection)
        .map_err(|e| DatabaseInitError { side: Box::new(e) })?;

    log::debug!("Initializing model-specific data inside the database");
    model_init(&connection).map_err(|e| DatabaseInitError { side: Box::new(e) })?;
    libs_to_db(libs, &connection)?;

    Ok((client, Mutex::new(connection)))
}

/// Writes the library information to the database.
///
/// # Arguments
///
/// * `libs` - A collection of peripheral libraries that have been loaded into memory
/// * `db` - A connection to the database
fn libs_to_db(libs: &Vec<TSLibrary>, db: &redis::Connection) -> Result<(), DatabaseInitError> {
    log::info!("Writing peripheral library information to the database");

    let mut lib_json: String;
    for lib in libs.iter() {
        // There's no reason for the daemon to continue if it cannot grab a lock on the libraries.
        let lib = lib.lock().expect("Could not obtain a lock on the library");
        lib_json =
            serde_json::to_string(&(*lib)).map_err(|e| DatabaseInitError { side: Box::new(e) })?;

        log::debug!("Writing {} to key libraries:{}", &lib_json, &lib.id());
        redis::cmd("SET")
            .arg(format!("libraries:{}", &lib.id()))
            .arg(format!("{}", &lib_json))
            .query(db)
            .map_err(|e| DatabaseInitError { side: Box::new(e) })?;
        Library::incr(&db).map_err(|e| DatabaseInitError { side: Box::new(e) })?;
    }

    Ok(())
}

/// Raised when an error occurs during database initialization.
#[derive(Debug)]
pub struct DatabaseInitError {
    side: Box<dyn Error>,
}

impl Error for DatabaseInitError {
    fn description(&self) -> &str {
        "Failed to initialize the database"
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.side)
    }
}

impl fmt::Display for DatabaseInitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DatabaseInitError {{ Cause: {} }}", &*self.side)
    }
}
