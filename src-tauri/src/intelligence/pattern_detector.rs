// Finds patterns in your command history
//
// Like when you always run "git add ." then "git commit" then "git push"
// Or when you keep running the same 3 docker commands in order

use crate::db::{Command, Database, PatternType};
use crate::error::Result;
use std::collections::HashMap;
use std::sync::Arc;

// Need to see something at least 3 times before calling it a pattern
const MIN_PATTERN_OCCURRENCES: usize = 3;

// Only save patterns we're at least 60% confident about
const MIN_CONFIDENCE: f64 = 0.6;

#[derive(Debug, Clone)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub commands: Vec<String>,
    pub confidence: f64,
    pub occurrences: usize,
    pub project_path: Option<String>,
}

pub struct PatternDetector {
    db: Arc<Database>,
}

impl PatternDetector {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    // Main function - finds all patterns in your history
    pub async fn detect_patterns(&self, project_path: Option<&str>) -> Result<Vec<Pattern>> {
        let mut patterns = Vec::new();

        // Find command sequences (A -> B -> C)
        let sequential = self.detect_sequential_patterns(project_path).await?;
        patterns.extend(sequential);

        // Find frequently repeated combos
        let frequency = self.detect_frequency_patterns(project_path).await?;
        patterns.extend(frequency);

        // Only keep the good ones and save to db
        for pattern in &patterns {
            if pattern.confidence >= MIN_CONFIDENCE {
                let metadata = serde_json::json!({
                    "detected_at": chrono::Utc::now().to_rfc3339(),
                    "method": "auto"
                });

                let _ = self
                    .db
                    .store_pattern(
                        pattern.pattern_type.clone(),
                        pattern.commands.clone(),
                        pattern.project_path.clone(),
                        pattern.confidence,
                        metadata,
                    )
                    .await;
            }
        }

        Ok(patterns)
    }

    /// Detect sequential patterns (commands that follow each other)
    ///
    /// Uses sliding window algorithm to find command sequences
    async fn detect_sequential_patterns(&self, project_path: Option<&str>) -> Result<Vec<Pattern>> {
        let commands = self.db.get_recent_commands(project_path, 1000).await?;

        if commands.len() < 3 {
            return Ok(Vec::new());
        }

        let mut patterns = Vec::new();
        let window_sizes = [2, 3, 4, 5]; // Different sequence lengths

        for window_size in window_sizes {
            let sequences = self.extract_sequences(&commands, window_size);
            let pattern_candidates = self.find_frequent_sequences(sequences, window_size);

            patterns.extend(pattern_candidates);
        }

        Ok(patterns)
    }

    /// Extract command sequences using sliding window
    fn extract_sequences(&self, commands: &[Command], window_size: usize) -> Vec<Vec<String>> {
        let mut sequences = Vec::new();

        for window in commands.windows(window_size) {
            let sequence: Vec<String> = window.iter().map(|c| c.command.clone()).collect();
            sequences.push(sequence);
        }

        sequences
    }

    /// Find frequent sequences and calculate confidence
    fn find_frequent_sequences(&self, sequences: Vec<Vec<String>>, window_size: usize) -> Vec<Pattern> {
        let mut sequence_counts: HashMap<Vec<String>, usize> = HashMap::new();

        // Count occurrences
        for seq in sequences {
            *sequence_counts.entry(seq).or_insert(0) += 1;
        }

        // Filter and create patterns
        sequence_counts
            .into_iter()
            .filter(|(_, count)| *count >= MIN_PATTERN_OCCURRENCES)
            .map(|(commands, occurrences)| {
                let confidence = self.calculate_sequence_confidence(occurrences, window_size);

                Pattern {
                    pattern_type: PatternType::Sequential,
                    commands,
                    confidence,
                    occurrences,
                    project_path: None,
                }
            })
            .collect()
    }

    /// Calculate confidence score for sequential patterns
    ///
    /// Confidence increases with:
    /// - Number of occurrences
    /// - Sequence length
    fn calculate_sequence_confidence(&self, occurrences: usize, window_size: usize) -> f64 {
        let base_confidence = (occurrences as f64 / 10.0).min(0.7);
        let length_bonus = (window_size as f64 / 10.0).min(0.3);

        (base_confidence + length_bonus).min(1.0)
    }

    /// Detect frequency patterns (commonly used command groups)
    async fn detect_frequency_patterns(&self, project_path: Option<&str>) -> Result<Vec<Pattern>> {
        let commands = self.db.get_most_used_commands(project_path, 50).await?;

        let mut patterns = Vec::new();

        // Group commands by category (git, npm, docker, etc.)
        let categories = self.categorize_commands(&commands);

        for (_category, cmds) in categories {
            if cmds.len() >= 3 {
                let total_usage: i32 = cmds.iter().map(|c| c.usage_count).sum();
                let avg_usage = total_usage as f64 / cmds.len() as f64;

                // High usage = high confidence
                let confidence = (avg_usage / 20.0).min(0.95);

                if confidence >= MIN_CONFIDENCE {
                    patterns.push(Pattern {
                        pattern_type: PatternType::Frequency,
                        commands: cmds.iter().map(|c| c.command.clone()).collect(),
                        confidence,
                        occurrences: total_usage as usize,
                        project_path: project_path.map(|s| s.to_string()),
                    });
                }
            }
        }

        Ok(patterns)
    }

    /// Categorize commands by their primary tool (git, npm, docker, etc.)
    fn categorize_commands(&self, commands: &[Command]) -> HashMap<String, Vec<Command>> {
        let mut categories: HashMap<String, Vec<Command>> = HashMap::new();

        for cmd in commands {
            let category = self.extract_category(&cmd.command);
            categories
                .entry(category)
                .or_insert_with(Vec::new)
                .push(cmd.clone());
        }

        categories
    }

    /// Extract category from command (first word)
    fn extract_category(&self, command: &str) -> String {
        command
            .split_whitespace()
            .next()
            .unwrap_or("other")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::CommandInput;

    async fn setup() -> PatternDetector {
        let db = Arc::new(Database::new_test().await.unwrap());

        // Insert test sequences
        let test_commands = vec![
            "git add .",
            "git commit -m 'test'",
            "git push",
            "git add .",
            "git commit -m 'fix'",
            "git push",
            "git add .",
            "git commit -m 'update'",
            "git push",
        ];

        for cmd in test_commands {
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

        PatternDetector::new(db)
    }

    #[tokio::test]
    async fn test_detect_sequential_patterns() {
        let detector = setup().await;

        let patterns = detector.detect_patterns(Some("/test")).await.unwrap();

        // Should detect git add -> commit -> push sequence
        assert!(!patterns.is_empty());

        let sequential: Vec<_> = patterns
            .iter()
            .filter(|p| matches!(p.pattern_type, PatternType::Sequential))
            .collect();

        assert!(!sequential.is_empty());
    }

    #[tokio::test]
    async fn test_extract_category() {
        let detector = setup().await;

        assert_eq!(detector.extract_category("git add ."), "git");
        assert_eq!(detector.extract_category("npm install"), "npm");
        assert_eq!(detector.extract_category("docker ps"), "docker");
    }

    #[tokio::test]
    async fn test_confidence_calculation() {
        let detector = setup().await;

        let confidence = detector.calculate_sequence_confidence(5, 3);
        assert!(confidence >= MIN_CONFIDENCE);
        assert!(confidence <= 1.0);
    }
}
