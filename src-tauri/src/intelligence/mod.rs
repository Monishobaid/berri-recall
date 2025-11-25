/// Intelligence module
///
/// Handles pattern detection and smart suggestions based on command history.

pub mod analyzer;
pub mod context_detector;
pub mod pattern_detector;
pub mod scorer;
pub mod suggestion_engine;

pub use analyzer::Analyzer;
pub use context_detector::{Context, ContextDetector, DayOfWeek, ProjectType, TimeOfDay};
pub use pattern_detector::{Pattern, PatternDetector};
pub use scorer::Scorer;
pub use suggestion_engine::{SmartSuggestion, SuggestionEngine};
