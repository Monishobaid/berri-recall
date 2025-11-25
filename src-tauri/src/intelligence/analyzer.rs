/// Main analyzer orchestrator
///
/// Coordinates pattern detection and suggestion generation.

use crate::db::Database;
use crate::error::Result;
use crate::intelligence::{PatternDetector, SuggestionEngine};
use std::sync::Arc;

/// Main analyzer
pub struct Analyzer {
    pattern_detector: PatternDetector,
    suggestion_engine: SuggestionEngine,
}

impl Analyzer {
    /// Create a new analyzer
    pub fn new(db: Arc<Database>) -> Self {
        let pattern_detector = PatternDetector::new(Arc::clone(&db));
        let suggestion_engine = SuggestionEngine::new(db);

        Self {
            pattern_detector,
            suggestion_engine,
        }
    }

    /// Run full analysis
    ///
    /// Detects patterns and generates suggestions
    pub async fn analyze(&self, project_path: Option<&str>) -> Result<AnalysisReport> {
        // Detect patterns
        let patterns = self.pattern_detector.detect_patterns(project_path).await?;

        // Generate suggestions
        let suggestions = self.suggestion_engine.generate_suggestions().await?;

        Ok(AnalysisReport {
            patterns_found: patterns.len(),
            suggestions_generated: suggestions.len(),
            patterns,
            suggestions,
        })
    }
}

/// Analysis report
#[derive(Debug)]
pub struct AnalysisReport {
    pub patterns_found: usize,
    pub suggestions_generated: usize,
    pub patterns: Vec<crate::intelligence::Pattern>,
    pub suggestions: Vec<crate::intelligence::SmartSuggestion>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::CommandInput;

    async fn setup() -> Analyzer {
        let db = Arc::new(Database::new_test().await.unwrap());

        // Insert test commands
        for _ in 0..3 {
            for cmd in &["git add .", "git commit -m 'test'", "git push"] {
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
        }

        Analyzer::new(db)
    }

    #[tokio::test]
    async fn test_analyze() {
        let analyzer = setup().await;

        let report = analyzer.analyze(Some("/test")).await.unwrap();

        // Should find patterns and generate suggestions
        assert!(report.patterns_found > 0 || report.suggestions_generated > 0);
    }
}
