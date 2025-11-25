/// Suggestion engine
///
/// Generates smart command suggestions based on patterns and context.

use crate::db::{Database, Suggestion};
use crate::error::Result;
use crate::intelligence::{Context, ContextDetector, PatternDetector};
use std::sync::Arc;

/// Suggestion with reasoning
#[derive(Debug, Clone)]
pub struct SmartSuggestion {
    pub command: String,
    pub reason: String,
    pub confidence: f64,
}

/// Suggestion engine
pub struct SuggestionEngine {
    db: Arc<Database>,
    pattern_detector: PatternDetector,
}

impl SuggestionEngine {
    /// Create a new suggestion engine
    pub fn new(db: Arc<Database>) -> Self {
        let pattern_detector = PatternDetector::new(Arc::clone(&db));

        Self {
            db,
            pattern_detector,
        }
    }

    /// Generate suggestions for current context
    ///
    /// # Returns
    /// * `Ok(Vec<SmartSuggestion>)` - List of suggestions with reasoning
    pub async fn generate_suggestions(&self) -> Result<Vec<SmartSuggestion>> {
        let context = ContextDetector::detect()?;
        let mut suggestions = Vec::new();

        // Get suggestions from patterns
        let pattern_suggestions = self.suggest_from_patterns(&context).await?;
        suggestions.extend(pattern_suggestions);

        // Get context-based suggestions
        let context_suggestions = self.suggest_from_context(&context).await?;
        suggestions.extend(context_suggestions);

        // Get time-based suggestions
        let time_suggestions = self.suggest_from_time(&context).await?;
        suggestions.extend(time_suggestions);

        // Sort by confidence
        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        // Take top 5
        suggestions.truncate(5);

        // Store suggestions in database
        for suggestion in &suggestions {
            let _ = self
                .db
                .store_suggestion(
                    context.working_directory.clone(),
                    Some(format!("{:?}", context.time_of_day)),
                    suggestion.command.clone(),
                    Some(suggestion.reason.clone()),
                    suggestion.confidence,
                )
                .await;
        }

        Ok(suggestions)
    }

    /// Generate suggestions based on detected patterns
    async fn suggest_from_patterns(&self, context: &Context) -> Result<Vec<SmartSuggestion>> {
        let patterns = self
            .pattern_detector
            .detect_patterns(Some(&context.working_directory))
            .await?;

        let mut suggestions = Vec::new();

        for pattern in patterns {
            if pattern.commands.len() >= 2 {
                // Get recent commands to see what was just executed
                let recent = self
                    .db
                    .get_recent_commands(Some(&context.working_directory), 5)
                    .await?;

                if let Some(last_cmd) = recent.first() {
                    // Check if last command matches start of pattern
                    if let Some(next_cmd) = self.predict_next_in_sequence(&last_cmd.command, &pattern.commands) {
                        suggestions.push(SmartSuggestion {
                            command: next_cmd.clone(),
                            reason: format!(
                                "You usually run '{}' after '{}'",
                                next_cmd, last_cmd.command
                            ),
                            confidence: pattern.confidence,
                        });
                    }
                }
            }
        }

        Ok(suggestions)
    }

    /// Predict next command in a sequence
    fn predict_next_in_sequence(&self, last_cmd: &str, sequence: &[String]) -> Option<String> {
        for (i, cmd) in sequence.iter().enumerate() {
            if cmd == last_cmd && i + 1 < sequence.len() {
                return Some(sequence[i + 1].clone());
            }
        }
        None
    }

    /// Generate context-based suggestions
    async fn suggest_from_context(&self, context: &Context) -> Result<Vec<SmartSuggestion>> {
        let mut suggestions = Vec::new();

        // Suggest based on project type
        if let Some(project_type) = &context.project_type {
            let type_suggestions = match project_type {
                crate::intelligence::ProjectType::Node => vec![
                    SmartSuggestion {
                        command: "npm install".to_string(),
                        reason: "Node project: install dependencies".to_string(),
                        confidence: 0.7,
                    },
                    SmartSuggestion {
                        command: "npm test".to_string(),
                        reason: "Node project: run tests".to_string(),
                        confidence: 0.65,
                    },
                ],
                crate::intelligence::ProjectType::Rust => vec![
                    SmartSuggestion {
                        command: "cargo build".to_string(),
                        reason: "Rust project: build project".to_string(),
                        confidence: 0.7,
                    },
                    SmartSuggestion {
                        command: "cargo test".to_string(),
                        reason: "Rust project: run tests".to_string(),
                        confidence: 0.65,
                    },
                ],
                crate::intelligence::ProjectType::Python => vec![
                    SmartSuggestion {
                        command: "pip install -r requirements.txt".to_string(),
                        reason: "Python project: install dependencies".to_string(),
                        confidence: 0.7,
                    },
                    SmartSuggestion {
                        command: "python -m pytest".to_string(),
                        reason: "Python project: run tests".to_string(),
                        confidence: 0.65,
                    },
                ],
                _ => vec![],
            };

            suggestions.extend(type_suggestions);
        }

        // Suggest based on git branch
        if let Some(branch) = &context.git_branch {
            if branch.contains("feature") || branch.contains("feat") {
                suggestions.push(SmartSuggestion {
                    command: "git push".to_string(),
                    reason: format!("On feature branch '{}': push changes", branch),
                    confidence: 0.6,
                });
            }
        }

        Ok(suggestions)
    }

    /// Generate time-based suggestions
    async fn suggest_from_time(&self, context: &Context) -> Result<Vec<SmartSuggestion>> {
        let mut suggestions = Vec::new();

        // Monday morning suggestions
        if matches!(
            context.day_of_week,
            crate::intelligence::DayOfWeek::Monday
        ) && matches!(
            context.time_of_day,
            crate::intelligence::TimeOfDay::Morning
        ) {
            suggestions.push(SmartSuggestion {
                command: "git pull".to_string(),
                reason: "Monday morning: sync with latest changes".to_string(),
                confidence: 0.65,
            });
        }

        // Friday afternoon suggestions
        if matches!(
            context.day_of_week,
            crate::intelligence::DayOfWeek::Friday
        ) && matches!(
            context.time_of_day,
            crate::intelligence::TimeOfDay::Afternoon
        ) {
            suggestions.push(SmartSuggestion {
                command: "git status".to_string(),
                reason: "Friday afternoon: check for uncommitted changes".to_string(),
                confidence: 0.6,
            });
        }

        Ok(suggestions)
    }

    /// Get existing suggestions from database
    pub async fn get_suggestions(&self, project_path: &str) -> Result<Vec<Suggestion>> {
        self.db.get_suggestions(project_path, None).await
    }

    /// Record feedback on a suggestion
    pub async fn record_feedback(&self, suggestion_id: i64, accepted: bool) -> Result<()> {
        self.db.record_suggestion_feedback(suggestion_id, accepted).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::CommandInput;

    async fn setup() -> SuggestionEngine {
        let db = Arc::new(Database::new_test().await.unwrap());

        // Insert test data
        let commands = vec!["git add .", "git commit -m 'test'", "git push"];

        for cmd in commands {
            db.record_command(CommandInput {
                project_path: "/test".to_string(),
                command: cmd.to_string(),
                execution_time_ms: None,
                exit_code: Some(0),
                context: None,
            })
            .await
            .unwrap();
        }

        SuggestionEngine::new(db)
    }

    #[tokio::test]
    async fn test_generate_suggestions() {
        let engine = setup().await;

        let suggestions = engine.generate_suggestions().await.unwrap();

        // Should generate at least some suggestions
        assert!(!suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_predict_next_in_sequence() {
        let engine = setup().await;

        let sequence = vec!["git add .".to_string(), "git commit".to_string(), "git push".to_string()];

        let next = engine.predict_next_in_sequence("git add .", &sequence);
        assert_eq!(next, Some("git commit".to_string()));

        let next2 = engine.predict_next_in_sequence("git commit", &sequence);
        assert_eq!(next2, Some("git push".to_string()));
    }
}
