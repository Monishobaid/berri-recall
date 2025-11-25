/// Data models for database entities
///
/// All models map to database tables and use sqlx for type-safe queries.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents a recorded command
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Command {
    pub id: i64,
    pub project_path: String,
    pub command: String,
    pub timestamp: String, // ISO 8601 format from SQLite
    pub is_fav: bool,
    pub usage_count: i32,
    pub execution_time_ms: Option<i32>,
    pub exit_code: Option<i32>,
    pub tags: Option<String>, // JSON array
    pub context: Option<String>,
}

impl Command {
    /// Parse tags from JSON string
    pub fn get_tags(&self) -> Vec<String> {
        self.tags
            .as_ref()
            .and_then(|t| serde_json::from_str(t).ok())
            .unwrap_or_default()
    }

    /// Set tags as JSON string
    pub fn set_tags(&mut self, tags: Vec<String>) -> Result<(), serde_json::Error> {
        self.tags = Some(serde_json::to_string(&tags)?);
        Ok(())
    }
}

/// Input for recording a new command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInput {
    pub project_path: String,
    pub command: String,
    pub execution_time_ms: Option<i32>,
    pub exit_code: Option<i32>,
    pub context: Option<String>,
}

/// Detected command pattern
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CommandPattern {
    pub id: i64,
    pub pattern_type: String, // 'sequence', 'frequency', 'time_based', 'context_based'
    pub commands: String,     // JSON array
    pub project_path: Option<String>,
    pub confidence_score: f64,
    pub occurrences: i32,
    pub last_seen: String, // ISO 8601 format from SQLite
    pub metadata: Option<String>, // JSON
}

impl CommandPattern {
    /// Get commands from JSON
    pub fn get_commands(&self) -> Vec<String> {
        serde_json::from_str(&self.commands).unwrap_or_default()
    }

    /// Get metadata from JSON
    pub fn get_metadata(&self) -> serde_json::Value {
        self.metadata
            .as_ref()
            .and_then(|m| serde_json::from_str(m).ok())
            .unwrap_or(serde_json::json!({}))
    }
}

/// Pattern types enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PatternType {
    Sequential,
    Frequency,
    TimeBased,
    ContextBased,
}

impl std::fmt::Display for PatternType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PatternType::Sequential => "sequence",
            PatternType::Frequency => "frequency",
            PatternType::TimeBased => "time_based",
            PatternType::ContextBased => "context_based",
        };
        write!(f, "{}", s)
    }
}

/// Command suggestion
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Suggestion {
    pub id: i64,
    pub project_path: String,
    pub context: Option<String>,
    pub suggested_command: String,
    pub reason: Option<String>,
    pub confidence: f64,
    pub times_accepted: i32,
    pub times_rejected: i32,
    pub created_at: String, // ISO 8601 format from SQLite
    pub last_suggested: Option<String>, // ISO 8601 format from SQLite
}

impl Suggestion {
    /// Calculate acceptance rate
    pub fn acceptance_rate(&self) -> f64 {
        let total = self.times_accepted + self.times_rejected;
        if total == 0 {
            0.0
        } else {
            self.times_accepted as f64 / total as f64
        }
    }
}

/// User preference
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Preference {
    pub key: String,
    pub value: String,
}

/// Command alias
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Alias {
    pub alias: String,
    pub command: String,
    pub project_path: Option<String>,
    pub created_at: String, // ISO 8601 format from SQLite
}

/// Execution context for a command
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExecutionContext {
    pub id: i64,
    pub command_id: i64,
    pub working_directory: Option<String>,
    pub previous_command: Option<String>,
    pub time_of_day: Option<String>,
    pub day_of_week: Option<String>,
    pub git_branch: Option<String>,
    pub files_changed: Option<String>, // JSON array
}

impl ExecutionContext {
    /// Get files changed from JSON
    pub fn get_files_changed(&self) -> Vec<String> {
        self.files_changed
            .as_ref()
            .and_then(|f| serde_json::from_str(f).ok())
            .unwrap_or_default()
    }
}

/// Search results with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub command: Command,
    pub score: f64, // Fuzzy match score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_tags() {
        let mut cmd = Command {
            id: 1,
            project_path: "/test".to_string(),
            command: "ls".to_string(),
            timestamp: "2025-11-25T00:00:00Z".to_string(),
            is_fav: false,
            usage_count: 1,
            execution_time_ms: None,
            exit_code: None,
            tags: None,
            context: None,
        };

        cmd.set_tags(vec!["git".to_string(), "test".to_string()])
            .unwrap();
        let tags = cmd.get_tags();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"git".to_string()));
    }

    #[test]
    fn test_suggestion_acceptance_rate() {
        let suggestion = Suggestion {
            id: 1,
            project_path: "/test".to_string(),
            context: None,
            suggested_command: "npm test".to_string(),
            reason: None,
            confidence: 0.8,
            times_accepted: 8,
            times_rejected: 2,
            created_at: "2025-11-25T00:00:00Z".to_string(),
            last_suggested: None,
        };

        assert_eq!(suggestion.acceptance_rate(), 0.8);
    }

    #[test]
    fn test_pattern_type_display() {
        assert_eq!(PatternType::Sequential.to_string(), "sequence");
        assert_eq!(PatternType::TimeBased.to_string(), "time_based");
    }
}
