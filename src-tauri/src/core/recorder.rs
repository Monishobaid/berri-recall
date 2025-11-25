// Records commands to the database
//
// Filters out sensitive stuff like passwords and API keys

use crate::db::{CommandInput, Database};
use crate::error::{RecallError, Result};
use regex::Regex;
use std::sync::Arc;

// Don't let anyone record a 10MB command. that's just weird.
const MAX_COMMAND_LENGTH: usize = 10_000;

// Regex patterns for stuff we definitely shouldn't record
const SENSITIVE_PATTERNS: &[&str] = &[
    r"password\s*=",
    r"pwd\s*=",
    r"passwd\s*=",
    r"token\s*=",
    r"api[_-]?key\s*=",
    r"secret\s*=",
    r"auth\s*=",
    r"bearer\s+",
    r"--password",
    r"--token",
    r"-p\s+\S+", // -p with a password right after it
];

pub struct Recorder {
    db: Arc<Database>,
    sensitive_regex: Vec<Regex>,
}

impl Recorder {
    pub fn new(db: Arc<Database>) -> Self {
        // Build all the regex patterns once so we don't recompile them every time
        let sensitive_regex = SENSITIVE_PATTERNS
            .iter()
            .filter_map(|pattern| Regex::new(pattern).ok())
            .collect();

        Self {
            db,
            sensitive_regex,
        }
    }

    // Main recording function. Checks if the command is safe, cleans it up, saves it.
    pub async fn record(
        &self,
        command: &str,
        project_path: &str,
        execution_time_ms: Option<i32>,
        exit_code: Option<i32>,
        context: Option<String>,
    ) -> Result<i64> {
        // Make sure it's safe to record
        self.validate_command(command)?;

        // Clean up any weird characters
        let sanitized = self.sanitize_command(command);

        let input = CommandInput {
            project_path: project_path.to_string(),
            command: sanitized,
            execution_time_ms,
            exit_code,
            context,
        };

        // Shove it in the database
        let id = self.db.record_command(input).await?;

        Ok(id)
    }

    // Check if this command is safe to record (not empty, not huge, no passwords)
    fn validate_command(&self, command: &str) -> Result<()> {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return Err(RecallError::InvalidCommand("empty command".to_string()));
        }

        // Nobody needs a 10KB command
        if trimmed.len() > MAX_COMMAND_LENGTH {
            return Err(RecallError::CommandTooLong(MAX_COMMAND_LENGTH));
        }

        // Check for sensitive data
        if self.contains_sensitive_data(trimmed) {
            return Err(RecallError::SensitiveData);
        }

        Ok(())
    }

    /// Sanitize a command string
    ///
    /// - Removes null bytes
    /// - Trims whitespace
    /// - Normalizes whitespace (multiple spaces to single)
    fn sanitize_command(&self, command: &str) -> String {
        command
            .replace('\0', "") // Remove null bytes
            .trim() // Trim edges
            .split_whitespace() // Split on whitespace
            .collect::<Vec<_>>() // Collect parts
            .join(" ") // Join with single space
    }

    /// Check if command contains sensitive data
    ///
    /// Uses regex patterns to detect passwords, tokens, etc.
    fn contains_sensitive_data(&self, command: &str) -> bool {
        let lowercase = command.to_lowercase();

        self.sensitive_regex
            .iter()
            .any(|regex| regex.is_match(&lowercase))
    }

    /// Check if a command should be ignored
    ///
    /// Some commands are not useful to remember:
    /// - Very short commands (single char)
    /// - Common navigation commands
    /// - History commands
    pub fn should_ignore(&self, command: &str) -> bool {
        let trimmed = command.trim();

        // Too short
        if trimmed.len() < 2 {
            return true;
        }

        // Ignore list
        let ignore_list = [
            "ls",
            "cd",
            "pwd",
            "exit",
            "clear",
            "history",
            "recall",
        ];

        ignore_list.contains(&trimmed)
    }

    /// Batch record multiple commands
    ///
    /// Useful for importing history.
    pub async fn record_batch(
        &self,
        commands: Vec<(String, String)>, // (command, project_path)
    ) -> Result<Vec<i64>> {
        let mut ids = Vec::new();

        for (command, project_path) in commands {
            match self.record(&command, &project_path, None, None, None).await {
                Ok(id) => ids.push(id),
                Err(e) => {
                    // Log error but continue with other commands
                    eprintln!("Failed to record '{}': {}", command, e);
                }
            }
        }

        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_recorder() -> Recorder {
        let db = Database::new_test().await.unwrap();
        Recorder::new(Arc::new(db))
    }

    #[tokio::test]
    async fn test_record_valid_command() {
        let recorder = create_test_recorder().await;

        let id = recorder
            .record("npm test", "/test/project", None, None, None)
            .await
            .unwrap();

        assert!(id > 0);
    }

    #[tokio::test]
    async fn test_record_empty_command() {
        let recorder = create_test_recorder().await;

        let result = recorder.record("   ", "/test/project", None, None, None).await;

        assert!(result.is_err());
        match result {
            Err(RecallError::InvalidCommand(_)) => {}
            _ => panic!("Expected InvalidCommand error"),
        }
    }

    #[tokio::test]
    async fn test_record_sensitive_command() {
        let recorder = create_test_recorder().await;

        let result = recorder
            .record(
                "mysql -u root --password=secret123",
                "/test",
                None,
                None,
                None,
            )
            .await;

        assert!(result.is_err());
        match result {
            Err(RecallError::SensitiveData) => {}
            _ => panic!("Expected SensitiveData error"),
        }
    }

    #[tokio::test]
    async fn test_sanitize_command() {
        let db = Database::new_test().await.unwrap();
        let recorder = Recorder::new(Arc::new(db));

        let sanitized = recorder.sanitize_command("  npm    test   ");
        assert_eq!(sanitized, "npm test");

        let sanitized = recorder.sanitize_command("cmd\0with\0nulls");
        assert!(!sanitized.contains('\0'));
    }

    #[tokio::test]
    async fn test_should_ignore() {
        let recorder = create_test_recorder().await;

        assert!(recorder.should_ignore("ls"));
        assert!(recorder.should_ignore("cd"));
        assert!(recorder.should_ignore("exit"));
        assert!(!recorder.should_ignore("npm test"));
    }

    #[tokio::test]
    async fn test_contains_sensitive_data() {
        let db = Database::new_test().await.unwrap();
        let recorder = Recorder::new(Arc::new(db));

        assert!(recorder.contains_sensitive_data("export API_KEY=abc123"));
        assert!(recorder.contains_sensitive_data("curl -H 'Authorization: Bearer token'"));
        assert!(recorder.contains_sensitive_data("mysql -p secret"));
        assert!(!recorder.contains_sensitive_data("npm install"));
    }

    #[tokio::test]
    async fn test_command_too_long() {
        let recorder = create_test_recorder().await;

        let long_cmd = "a".repeat(MAX_COMMAND_LENGTH + 1);
        let result = recorder.record(&long_cmd, "/test", None, None, None).await;

        assert!(result.is_err());
        match result {
            Err(RecallError::CommandTooLong(_)) => {}
            _ => panic!("Expected CommandTooLong error"),
        }
    }

    #[tokio::test]
    async fn test_record_with_metadata() {
        let recorder = create_test_recorder().await;

        let id = recorder
            .record(
                "cargo build",
                "/test/project",
                Some(5000),
                Some(0),
                Some("after git pull".to_string()),
            )
            .await
            .unwrap();

        assert!(id > 0);
    }
}
