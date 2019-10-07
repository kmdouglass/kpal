//! Constant values that affect the operation of the daemon.
use std::time::Duration;

pub const ATTRIBUTE_NAME_BUFFER_LENGTH: usize = 512;
pub const DATABASE_INDEX: &str = "0";
pub const KPAL_DIR: &str = ".kpal";
pub const LIBRARY_DIR: &str = "libraries";
pub const SCHEDULER_SLEEP_DURATION: Duration = Duration::from_secs(1);
pub const TASK_INTERVAL_DURATION: Duration = Duration::from_secs(5);
