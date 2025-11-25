/// Database module for recall-cli
///
/// Handles all database operations using SQLite and sqlx.
/// Implements connection pooling for performance.

pub mod connection;
pub mod models;
pub mod queries;

pub use connection::Database;
pub use models::*;
