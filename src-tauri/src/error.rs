/// Error types for recall-cli
///
/// This module defines all possible errors that can occur in the application.
/// Uses thiserror for ergonomic error handling.

use thiserror::Error;

/// Main error type for recall-cli operations
#[derive(Error, Debug)]
pub enum RecallError {
    /// Database-related errors
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// I/O errors (file operations, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Git-related errors
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    /// Command not found in history
    #[error("Command not found: {0}")]
    CommandNotFound(String),

    /// Invalid command format or content
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    /// Project root detection failed
    #[error("Could not detect project root for path: {0}")]
    ProjectRootNotFound(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Pattern detection error
    #[error("Pattern detection error: {0}")]
    PatternDetection(String),

    /// Suggestion engine error
    #[error("Suggestion error: {0}")]
    Suggestion(String),

    /// Command contains sensitive data
    #[error("Command contains sensitive data and was not recorded")]
    SensitiveData,

    /// Command exceeds maximum length
    #[error("Command exceeds maximum allowed length of {0} characters")]
    CommandTooLong(usize),

    /// Generic error with message
    #[error("{0}")]
    Generic(String),
}

/// Result type alias for recall-cli operations
pub type Result<T> = std::result::Result<T, RecallError>;

/// Convert RecallError to a user-friendly error message
impl RecallError {
    pub fn user_message(&self) -> String {
        match self {
            RecallError::Database(e) => {
                format!("Database error occurred. Please try again. Details: {}", e)
            }
            RecallError::Io(e) => {
                format!("File system error. Check permissions. Details: {}", e)
            }
            RecallError::Git(e) => {
                format!("Git operation failed. Details: {}", e)
            }
            RecallError::CommandNotFound(cmd) => {
                format!("Command '{}' not found in history", cmd)
            }
            RecallError::InvalidCommand(reason) => {
                format!("Invalid command: {}", reason)
            }
            RecallError::ProjectRootNotFound(path) => {
                format!("Could not find project root for: {}", path)
            }
            RecallError::Config(msg) => {
                format!("Configuration issue: {}", msg)
            }
            RecallError::Serialization(e) => {
                format!("Data format error: {}", e)
            }
            RecallError::PatternDetection(msg) => {
                format!("Pattern detection failed: {}", msg)
            }
            RecallError::Suggestion(msg) => {
                format!("Suggestion generation failed: {}", msg)
            }
            RecallError::SensitiveData => {
                "Command contains sensitive data and was not recorded".to_string()
            }
            RecallError::CommandTooLong(max) => {
                format!("Command exceeds maximum length of {} characters", max)
            }
            RecallError::Generic(msg) => msg.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_user_messages() {
        let err = RecallError::CommandNotFound("test".to_string());
        assert!(err.user_message().contains("test"));

        let err = RecallError::SensitiveData;
        assert!(err.user_message().contains("sensitive"));
    }

    #[test]
    fn test_error_display() {
        let err = RecallError::InvalidCommand("empty command".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Invalid command"));
    }
}
