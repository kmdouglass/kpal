//! A JSON REST API integration for KPAL based on JSON.
mod errors;
mod handlers;
mod routes;
mod schemas;

pub use errors::{status_from_reason, RestIntegrationError};
pub use routes::routes;
