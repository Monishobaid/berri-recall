/// SQL query functions for database operations
///
/// All queries use sqlx for compile-time verification and type safety.

use crate::db::models::*;
use crate::db::Database;
use crate::error::Result;
use chrono::Utc;
use sqlx::Row;

impl Database {
    /// Record a new command or increment usage count if it exists
    ///
    /// # Arguments
    /// * `input` - Command input data
    ///
    /// # Returns
    /// * `Ok(i64)` - The command ID
    /// * `Err(RecallError)` - If database operation fails
    pub async fn record_command(&self, input: CommandInput) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO commands (project_path, command, execution_time_ms, exit_code, context)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(project_path, command) DO UPDATE SET
                usage_count = usage_count + 1,
                timestamp = CURRENT_TIMESTAMP,
                execution_time_ms = excluded.execution_time_ms,
                exit_code = excluded.exit_code
            RETURNING id
            "#,
        )
        .bind(&input.project_path)
        .bind(&input.command)
        .bind(input.execution_time_ms)
        .bind(input.exit_code)
        .bind(input.context)
        .fetch_one(self.pool())
        .await?;

        Ok(result.get(0))
    }

    /// Get recent commands for a project
    ///
    /// # Arguments
    /// * `project_path` - Optional project path filter (None for all projects)
    /// * `limit` - Maximum number of commands to return
    ///
    /// # Returns
    /// * `Ok(Vec<Command>)` - List of commands
    pub async fn get_recent_commands(
        &self,
        project_path: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Command>> {
        let commands = if let Some(path) = project_path {
            sqlx::query_as::<_, Command>(
                "SELECT * FROM commands WHERE project_path = ? ORDER BY timestamp DESC LIMIT ?",
            )
            .bind(path)
            .bind(limit)
            .fetch_all(self.pool())
            .await?
        } else {
            sqlx::query_as::<_, Command>(
                "SELECT * FROM commands ORDER BY timestamp DESC LIMIT ?",
            )
            .bind(limit)
            .fetch_all(self.pool())
            .await?
        };

        Ok(commands)
    }

    /// Get most used commands for a project
    ///
    /// # Arguments
    /// * `project_path` - Optional project path filter
    /// * `limit` - Maximum number of commands to return
    pub async fn get_most_used_commands(
        &self,
        project_path: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Command>> {
        let commands = if let Some(path) = project_path {
            sqlx::query_as::<_, Command>(
                "SELECT * FROM commands WHERE project_path = ? ORDER BY usage_count DESC LIMIT ?",
            )
            .bind(path)
            .bind(limit)
            .fetch_all(self.pool())
            .await?
        } else {
            sqlx::query_as::<_, Command>(
                "SELECT * FROM commands ORDER BY usage_count DESC LIMIT ?",
            )
            .bind(limit)
            .fetch_all(self.pool())
            .await?
        };

        Ok(commands)
    }

    /// Get favorite commands
    ///
    /// # Arguments
    /// * `project_path` - Optional project path filter
    pub async fn get_favorites(&self, project_path: Option<&str>) -> Result<Vec<Command>> {
        let commands = if let Some(path) = project_path {
            sqlx::query_as::<_, Command>(
                "SELECT * FROM commands WHERE project_path = ? AND is_fav = 1 ORDER BY usage_count DESC",
            )
            .bind(path)
            .fetch_all(self.pool())
            .await?
        } else {
            sqlx::query_as::<_, Command>(
                "SELECT * FROM commands WHERE is_fav = 1 ORDER BY usage_count DESC",
            )
            .fetch_all(self.pool())
            .await?
        };

        Ok(commands)
    }

    /// Toggle favorite status of a command
    ///
    /// # Arguments
    /// * `command_id` - ID of the command to toggle
    pub async fn toggle_favorite(&self, command_id: i64) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE commands SET is_fav = NOT is_fav WHERE id = ? RETURNING is_fav",
        )
        .bind(command_id)
        .fetch_one(self.pool())
        .await?;

        Ok(result.get(0))
    }

    /// Search commands by text (case-insensitive)
    ///
    /// # Arguments
    /// * `query` - Search query
    /// * `project_path` - Optional project path filter
    /// * `limit` - Maximum results
    pub async fn search_commands(
        &self,
        query: &str,
        project_path: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Command>> {
        let pattern = format!("%{}%", query);

        let commands = if let Some(path) = project_path {
            sqlx::query_as::<_, Command>(
                "SELECT * FROM commands WHERE project_path = ? AND command LIKE ? ORDER BY usage_count DESC LIMIT ?",
            )
            .bind(path)
            .bind(&pattern)
            .bind(limit)
            .fetch_all(self.pool())
            .await?
        } else {
            sqlx::query_as::<_, Command>(
                "SELECT * FROM commands WHERE command LIKE ? ORDER BY usage_count DESC LIMIT ?",
            )
            .bind(&pattern)
            .bind(limit)
            .fetch_all(self.pool())
            .await?
        };

        Ok(commands)
    }

    /// Get command by ID
    pub async fn get_command_by_id(&self, id: i64) -> Result<Option<Command>> {
        let command = sqlx::query_as::<_, Command>("SELECT * FROM commands WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await?;

        Ok(command)
    }

    /// Delete a command
    pub async fn delete_command(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM commands WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;

        Ok(())
    }

    /// Store a detected pattern
    pub async fn store_pattern(
        &self,
        pattern_type: PatternType,
        commands: Vec<String>,
        project_path: Option<String>,
        confidence: f64,
        metadata: serde_json::Value,
    ) -> Result<i64> {
        let commands_json = serde_json::to_string(&commands)?;
        let metadata_json = serde_json::to_string(&metadata)?;

        let result = sqlx::query(
            r#"
            INSERT INTO command_patterns (pattern_type, commands, project_path, confidence_score, metadata)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(pattern_type.to_string())
        .bind(commands_json)
        .bind(project_path)
        .bind(confidence)
        .bind(metadata_json)
        .fetch_one(self.pool())
        .await?;

        Ok(result.get(0))
    }

    /// Get patterns for a project
    pub async fn get_patterns(&self, project_path: Option<&str>) -> Result<Vec<CommandPattern>> {
        let patterns = if let Some(path) = project_path {
            sqlx::query_as::<_, CommandPattern>(
                "SELECT * FROM command_patterns WHERE project_path = ? OR project_path IS NULL ORDER BY confidence_score DESC",
            )
            .bind(path)
            .fetch_all(self.pool())
            .await?
        } else {
            sqlx::query_as::<_, CommandPattern>(
                "SELECT * FROM command_patterns ORDER BY confidence_score DESC",
            )
            .fetch_all(self.pool())
            .await?
        };

        Ok(patterns)
    }

    /// Store a suggestion
    pub async fn store_suggestion(
        &self,
        project_path: String,
        context: Option<String>,
        suggested_command: String,
        reason: Option<String>,
        confidence: f64,
    ) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO suggestions (project_path, context, suggested_command, reason, confidence)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(project_path)
        .bind(context)
        .bind(suggested_command)
        .bind(reason)
        .bind(confidence)
        .fetch_one(self.pool())
        .await?;

        Ok(result.get(0))
    }

    /// Get suggestions for a context
    pub async fn get_suggestions(
        &self,
        project_path: &str,
        context: Option<&str>,
    ) -> Result<Vec<Suggestion>> {
        let suggestions = if let Some(ctx) = context {
            sqlx::query_as::<_, Suggestion>(
                "SELECT * FROM suggestions WHERE project_path = ? AND context = ? ORDER BY confidence DESC",
            )
            .bind(project_path)
            .bind(ctx)
            .fetch_all(self.pool())
            .await?
        } else {
            sqlx::query_as::<_, Suggestion>(
                "SELECT * FROM suggestions WHERE project_path = ? ORDER BY confidence DESC",
            )
            .bind(project_path)
            .fetch_all(self.pool())
            .await?
        };

        Ok(suggestions)
    }

    /// Record suggestion feedback
    pub async fn record_suggestion_feedback(&self, id: i64, accepted: bool) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        if accepted {
            sqlx::query(
                "UPDATE suggestions SET times_accepted = times_accepted + 1, last_suggested = ? WHERE id = ?",
            )
            .bind(now)
            .bind(id)
            .execute(self.pool())
            .await?;
        } else {
            sqlx::query(
                "UPDATE suggestions SET times_rejected = times_rejected + 1, last_suggested = ? WHERE id = ?",
            )
            .bind(now)
            .bind(id)
            .execute(self.pool())
            .await?;
        }

        Ok(())
    }

    /// Get or set a preference
    pub async fn get_preference(&self, key: &str) -> Result<Option<String>> {
        let pref = sqlx::query_as::<_, Preference>("SELECT * FROM preferences WHERE key = ?")
            .bind(key)
            .fetch_optional(self.pool())
            .await?;

        Ok(pref.map(|p| p.value))
    }

    /// Set a preference
    pub async fn set_preference(&self, key: String, value: String) -> Result<()> {
        sqlx::query("INSERT OR REPLACE INTO preferences (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute(self.pool())
            .await?;

        Ok(())
    }

    /// Create an alias
    pub async fn create_alias(
        &self,
        alias: String,
        command: String,
        project_path: Option<String>,
    ) -> Result<()> {
        sqlx::query("INSERT OR REPLACE INTO aliases (alias, command, project_path) VALUES (?, ?, ?)")
            .bind(alias)
            .bind(command)
            .bind(project_path)
            .execute(self.pool())
            .await?;

        Ok(())
    }

    /// Get all aliases
    pub async fn get_aliases(&self, project_path: Option<&str>) -> Result<Vec<Alias>> {
        let aliases = if let Some(path) = project_path {
            sqlx::query_as::<_, Alias>(
                "SELECT * FROM aliases WHERE project_path = ? OR project_path IS NULL",
            )
            .bind(path)
            .fetch_all(self.pool())
            .await?
        } else {
            sqlx::query_as::<_, Alias>("SELECT * FROM aliases")
                .fetch_all(self.pool())
                .await?
        };

        Ok(aliases)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_and_retrieve_command() {
        let db = Database::new_test().await.unwrap();

        let input = CommandInput {
            project_path: "/test/project".to_string(),
            command: "npm test".to_string(),
            execution_time_ms: Some(1500),
            exit_code: Some(0),
            context: None,
        };

        let id = db.record_command(input).await.unwrap();
        assert!(id > 0);

        let cmd = db.get_command_by_id(id).await.unwrap();
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().command, "npm test");
    }

    #[tokio::test]
    async fn test_command_usage_increment() {
        let db = Database::new_test().await.unwrap();

        let input = CommandInput {
            project_path: "/test".to_string(),
            command: "ls -la".to_string(),
            execution_time_ms: None,
            exit_code: None,
            context: None,
        };

        // Record twice
        let id1 = db.record_command(input.clone()).await.unwrap();
        let id2 = db.record_command(input.clone()).await.unwrap();

        // Should be same ID (updated, not inserted)
        assert_eq!(id1, id2);

        let cmd = db.get_command_by_id(id1).await.unwrap().unwrap();
        assert_eq!(cmd.usage_count, 2);
    }

    #[tokio::test]
    async fn test_get_recent_commands() {
        let db = Database::new_test().await.unwrap();

        // Insert some commands
        for i in 1..=5 {
            let input = CommandInput {
                project_path: "/test".to_string(),
                command: format!("command{}", i),
                execution_time_ms: None,
                exit_code: None,
                context: None,
            };
            db.record_command(input).await.unwrap();
        }

        let recent = db.get_recent_commands(Some("/test"), 3).await.unwrap();
        assert_eq!(recent.len(), 3);
        // Most recent should be first
        assert_eq!(recent[0].command, "command5");
    }

    #[tokio::test]
    async fn test_toggle_favorite() {
        let db = Database::new_test().await.unwrap();

        let input = CommandInput {
            project_path: "/test".to_string(),
            command: "git status".to_string(),
            execution_time_ms: None,
            exit_code: None,
            context: None,
        };

        let id = db.record_command(input).await.unwrap();

        // Toggle on
        let is_fav = db.toggle_favorite(id).await.unwrap();
        assert_eq!(is_fav, true);

        // Toggle off
        let is_fav = db.toggle_favorite(id).await.unwrap();
        assert_eq!(is_fav, false);
    }

    #[tokio::test]
    async fn test_search_commands() {
        let db = Database::new_test().await.unwrap();

        let commands = vec!["npm install", "npm test", "cargo build"];
        for cmd in commands {
            let input = CommandInput {
                project_path: "/test".to_string(),
                command: cmd.to_string(),
                execution_time_ms: None,
                exit_code: None,
                context: None,
            };
            db.record_command(input).await.unwrap();
        }

        let results = db.search_commands("npm", Some("/test"), 10).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_preferences() {
        let db = Database::new_test().await.unwrap();

        db.set_preference("test_key".to_string(), "test_value".to_string())
            .await
            .unwrap();

        let value = db.get_preference("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));
    }
}
