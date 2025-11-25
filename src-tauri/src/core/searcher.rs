/// Command searcher with fuzzy matching
///
/// Provides fuzzy search capabilities for finding commands.

use crate::db::{Database, SearchResult};
use crate::error::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::sync::Arc;

/// Handles command searching with fuzzy matching
pub struct Searcher {
    db: Arc<Database>,
    matcher: SkimMatcherV2,
}

impl Searcher {
    /// Create a new searcher instance
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Search commands with fuzzy matching
    ///
    /// # Arguments
    /// * `query` - Search query
    /// * `project_path` - Optional project filter
    /// * `limit` - Maximum results to return
    ///
    /// # Returns
    /// * `Ok(Vec<SearchResult>)` - Search results sorted by score
    pub async fn search(
        &self,
        query: &str,
        project_path: Option<&str>,
        limit: i64,
    ) -> Result<Vec<SearchResult>> {
        // Get all commands (or use basic search as pre-filter)
        let commands = self.db.search_commands("", project_path, 1000).await?;

        // Apply fuzzy matching
        let mut results: Vec<SearchResult> = commands
            .into_iter()
            .filter_map(|cmd| {
                self.matcher
                    .fuzzy_match(&cmd.command, query)
                    .map(|score| SearchResult {
                        command: cmd,
                        score: score as f64,
                    })
            })
            .collect();

        // Sort by score (highest first)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Limit results
        results.truncate(limit as usize);

        Ok(results)
    }

    /// Search by tags
    pub async fn search_by_tags(
        &self,
        tags: Vec<String>,
        project_path: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let all_commands = self.db.get_recent_commands(project_path, 1000).await?;

        let results: Vec<SearchResult> = all_commands
            .into_iter()
            .filter(|cmd| {
                let cmd_tags = cmd.get_tags();
                tags.iter().any(|tag| cmd_tags.contains(tag))
            })
            .map(|cmd| SearchResult {
                command: cmd,
                score: 1.0,
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::CommandInput;

    async fn setup() -> Searcher {
        let db = Arc::new(Database::new_test().await.unwrap());

        // Insert test data
        let test_commands = vec!["npm install", "npm test", "cargo build", "git commit"];

        for cmd in test_commands {
            db.record_command(CommandInput {
                project_path: "/test".to_string(),
                command: cmd.to_string(),
                execution_time_ms: None,
                exit_code: None,
                context: None,
            })
            .await
            .unwrap();
        }

        Searcher::new(db)
    }

    #[tokio::test]
    async fn test_fuzzy_search() {
        let searcher = setup().await;

        let results = searcher.search("npm", Some("/test"), 10).await.unwrap();
        assert!(results.len() >= 2);
        assert!(results[0].command.command.contains("npm"));
    }

    #[tokio::test]
    async fn test_fuzzy_typo() {
        let searcher = setup().await;

        // Should still find "npm" even with typo
        let results = searcher.search("nmp", Some("/test"), 10).await.unwrap();
        assert!(!results.is_empty());
    }
}
