/// Command retriever
///
/// Handles fetching commands from database with various filters.

use crate::db::{Command, Database};
use crate::error::Result;
use std::sync::Arc;

/// Handles command retrieval operations
pub struct Retriever {
    db: Arc<Database>,
}

impl Retriever {
    /// Create a new retriever instance
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Get recent commands
    pub async fn get_recent(&self, project_path: Option<&str>, limit: i64) -> Result<Vec<Command>> {
        self.db.get_recent_commands(project_path, limit).await
    }

    /// Get most used commands
    pub async fn get_most_used(
        &self,
        project_path: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Command>> {
        self.db.get_most_used_commands(project_path, limit).await
    }

    /// Get favorite commands
    pub async fn get_favorites(&self, project_path: Option<&str>) -> Result<Vec<Command>> {
        self.db.get_favorites(project_path).await
    }

    /// Get command by ID
    pub async fn get_by_id(&self, id: i64) -> Result<Option<Command>> {
        self.db.get_command_by_id(id).await
    }

    /// Toggle favorite status
    pub async fn toggle_favorite(&self, id: i64) -> Result<bool> {
        self.db.toggle_favorite(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Recorder;
    use crate::db::CommandInput;

    async fn setup() -> (Retriever, Arc<Database>) {
        let db = Arc::new(Database::new_test().await.unwrap());
        let retriever = Retriever::new(Arc::clone(&db));
        (retriever, db)
    }

    #[tokio::test]
    async fn test_get_recent() {
        let (retriever, db) = setup().await;

        // Insert test data
        db.record_command(CommandInput {
            project_path: "/test".to_string(),
            command: "npm test".to_string(),
            execution_time_ms: None,
            exit_code: None,
            context: None,
        })
        .await
        .unwrap();

        let recent = retriever.get_recent(Some("/test"), 10).await.unwrap();
        assert_eq!(recent.len(), 1);
    }

    #[tokio::test]
    async fn test_toggle_favorite() {
        let (retriever, db) = setup().await;

        let id = db
            .record_command(CommandInput {
                project_path: "/test".to_string(),
                command: "git status".to_string(),
                execution_time_ms: None,
                exit_code: None,
                context: None,
            })
            .await
            .unwrap();

        let is_fav = retriever.toggle_favorite(id).await.unwrap();
        assert_eq!(is_fav, true);
    }
}
