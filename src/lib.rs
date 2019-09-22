pub mod constants {
    use std::time::Duration;

    pub const ATTRIBUTE_NAME_BUFFER_LENGTH: usize = 512;
    pub const DATABASE_INDEX: &str = "0";
    pub const KPAL_DIR: &str = ".kpal";
    pub const LIBRARY_DIR: &str = "peripherals";
    pub const SCHEDULER_SLEEP_DURATION: Duration = Duration::from_secs(1);
    pub const TASK_INTERVAL_DURATION: Duration = Duration::from_secs(5);
}
pub mod handlers;
pub mod init;
pub mod models;
pub mod plugins;
pub mod routes;
