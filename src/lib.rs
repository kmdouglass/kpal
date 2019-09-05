pub mod constants {
    pub const DATABASE_INDEX: &str = "0";
    pub const KPAL_DIR: &str = ".kpal";
    pub const LIBRARY_DIR: &str = "peripherals";
    pub const ATTRIBUTE_NAME_BUFFER_LENGTH: usize = 512;
}
pub mod handlers;
pub mod init;
pub mod models;
pub mod plugins;
pub mod routes;
