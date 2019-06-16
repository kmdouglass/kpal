use std::boxed::Box;
use std::error::Error;
use std::fmt;

use redis;
use rouille::Response;

use crate::models::Library;

pub fn get_libraries(db: &redis::Connection) -> Result<Response> {
    let libs_keys: Vec<String> = redis::cmd("KEYS")
        .arg("libraries:*")
        .query(db)
        .map_err(|e| RequestHandlerError { side: Box::new(e) })?;

    let libs_json: Vec<String> = redis::cmd("JSON.MGET")
        .arg(libs_keys)
        .arg(".")
        .query(db)
        .map_err(|e| RequestHandlerError { side: Box::new(e) })?;

    let mut result: Vec<Library> = Vec::new();
    for lib in libs_json.iter() {
        result.push(
            serde_json::from_str(&lib).map_err(|e| RequestHandlerError { side: Box::new(e) })?,
        );
    }

    Ok(Response::json(&result))
}

pub fn get_libraries_id(db: &redis::Connection, id: usize) -> Result<Response> {
    let result: String = redis::cmd("JSON.GET")
        .arg(format!("libraries:{}", &id))
        .arg(".")
        .query(db)
        .map_err(|e| RequestHandlerError { side: Box::new(e) })?;

    let result: Library =
        serde_json::from_str(&result).map_err(|e| RequestHandlerError { side: Box::new(e) })?;

    Ok(Response::json(&result))
}

pub type Result<T> = std::result::Result<T, RequestHandlerError>;

#[derive(Debug)]
pub struct RequestHandlerError {
    side: Box<dyn Error>,
}

impl Error for RequestHandlerError {
    fn description(&self) -> &str {
        "Failed to handle the HTTP request"
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.side)
    }
}

impl fmt::Display for RequestHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RequestHandlerError {{ Cause: {} }}", &*self.side)
    }
}
