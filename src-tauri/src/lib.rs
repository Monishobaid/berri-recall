/// recall-cli library
///
/// Core functionality for intelligent command memory system.

pub mod core;
pub mod db;
pub mod error;
pub mod intelligence;
pub mod shell;

// Re-exports for convenience
pub use db::Database;
pub use error::{RecallError, Result};
