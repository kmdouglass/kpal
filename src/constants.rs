//! Constant values that affect the operation of the daemon.
use std::time::Duration;

pub const ATTRIBUTE_NAME_BUFFER_LENGTH: usize = 512;
pub const DATABASE_INDEX: &str = "0";
pub const KPAL_DIR: &str = ".kpal";
pub const LIBRARY_DIR: &str = "libraries";
pub const REQUEST_TIMEOUT: Duration = Duration::from_millis(5000);
