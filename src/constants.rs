//! Constant values that affect the operation of the daemon.
use std::time::Duration;

/// The maximum length of a buffer that holds the C-string representing an attribute name.
pub const ATTRIBUTE_NAME_BUFFER_LENGTH: usize = 512;

/// The directory (relative to the user's HOME) that KPAL uses to store configuration files.
pub const KPAL_DIR: &str = ".kpal";

/// The directory (relative to the KPAL_DIR) that KPAL searches for plugin library files.
pub const LIBRARY_DIR: &str = "libraries";

/// The maximum amount of time that a request will wait before timing out in error.
pub const REQUEST_TIMEOUT: Duration = Duration::from_millis(5000);
