use std::boxed::Box;
use std::error::Error;
use std::fmt;
use std::sync::Mutex;

use log;
use redis;
use url::Url;

use crate::constants::DATABASE_INDEX;

// TODO Provide a connection pool rather than a single mutex to the database connection
pub fn init(db_addr: &Url) -> Result<Mutex<redis::Connection>, DatabaseInitError> {
    let mut db_addr = db_addr.clone();
    db_addr.set_path(DATABASE_INDEX);
    log::info!("Initializing the database connection to {}", db_addr);

    let connection = redis::Client::open(db_addr)
        .map_err(|e| DatabaseInitError { side: Box::new(e) })?
        .get_connection()
        .map_err(|e| DatabaseInitError { side: Box::new(e) })?;

    log::info!("Flushing database number {}", DATABASE_INDEX);
    redis::cmd("FLUSHDB")
        .query(&connection)
        .map_err(|e| DatabaseInitError { side: Box::new(e) })?;

    // TODO Add libraries to the database

    Ok(Mutex::new(connection))
}

#[derive(Debug)]
pub struct DatabaseInitError {
    side: Box<dyn Error>,
}

impl Error for DatabaseInitError {
    fn description(&self) -> &str {
        "Failed to initialze the database"
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.side)
    }
}

impl fmt::Display for DatabaseInitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DatabaseInitError {{ Cause {} }}", &*self.side)
    }
}
