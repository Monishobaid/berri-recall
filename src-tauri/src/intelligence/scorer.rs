/// Scoring algorithms for patterns and suggestions
///
/// Calculates confidence scores based on multiple factors.

/// Scorer for calculating confidence scores
pub struct Scorer;

impl Scorer {
    /// Calculate overall score for a suggestion
    ///
    /// # Arguments
    /// * `frequency` - How often the command is used (0.0-1.0)
    /// * `recency` - How recently it was used (0.0-1.0)
    /// * `pattern_confidence` - Confidence from pattern detection (0.0-1.0)
    /// * `context_match` - How well it matches current context (0.0-1.0)
    /// * `acceptance_rate` - Historical acceptance rate (0.0-1.0)
    ///
    /// # Returns
    /// * Score between 0.0 and 1.0
    pub fn calculate_suggestion_score(
        frequency: f64,
        recency: f64,
        pattern_confidence: f64,
        context_match: f64,
        acceptance_rate: f64,
    ) -> f64 {
        // Weighted average
        let score = frequency * 0.25
            + recency * 0.20
            + pattern_confidence * 0.25
            + context_match * 0.20
            + acceptance_rate * 0.10;

        score.clamp(0.0, 1.0)
    }

    /// Calculate frequency weight
    ///
    /// # Arguments
    /// * `usage_count` - Number of times used
    /// * `max_count` - Maximum usage count in dataset
    pub fn calculate_frequency_weight(usage_count: i32, max_count: i32) -> f64 {
        if max_count == 0 {
            return 0.0;
        }

        (usage_count as f64 / max_count as f64).clamp(0.0, 1.0)
    }

    /// Calculate recency weight using exponential decay
    ///
    /// # Arguments
    /// * `days_ago` - Number of days since last use
    pub fn calculate_recency_weight(days_ago: f64) -> f64 {
        // Exponential decay: newer = higher score
        // Half-life of 7 days
        let half_life = 7.0;
        (-days_ago / half_life * 2.0_f64.ln()).exp()
    }

    /// Calculate context match score
    ///
    /// # Arguments
    /// * `factors_matched` - Number of context factors that match
    /// * `total_factors` - Total number of context factors
    pub fn calculate_context_match(factors_matched: usize, total_factors: usize) -> f64 {
        if total_factors == 0 {
            return 0.0;
        }

        (factors_matched as f64 / total_factors as f64).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_suggestion_score() {
        let score = Scorer::calculate_suggestion_score(0.8, 0.9, 0.7, 0.6, 0.5);

        assert!(score > 0.0);
        assert!(score <= 1.0);
        assert!(score > 0.5); // With these high values, score should be decent
    }

    #[test]
    fn test_frequency_weight() {
        assert_eq!(Scorer::calculate_frequency_weight(5, 10), 0.5);
        assert_eq!(Scorer::calculate_frequency_weight(10, 10), 1.0);
        assert_eq!(Scorer::calculate_frequency_weight(0, 10), 0.0);
    }

    #[test]
    fn test_recency_weight() {
        // Recently used (1 day ago) should have high score
        let recent = Scorer::calculate_recency_weight(1.0);
        assert!(recent > 0.8);

        // Long time ago (30 days) should have low score
        let old = Scorer::calculate_recency_weight(30.0);
        assert!(old < 0.3);

        // Today (0 days) should be 1.0
        let today = Scorer::calculate_recency_weight(0.0);
        assert_eq!(today, 1.0);
    }

    #[test]
    fn test_context_match() {
        assert_eq!(Scorer::calculate_context_match(3, 5), 0.6);
        assert_eq!(Scorer::calculate_context_match(5, 5), 1.0);
        assert_eq!(Scorer::calculate_context_match(0, 5), 0.0);
    }
}
