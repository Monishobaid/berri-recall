/// Context detection for smart suggestions
///
/// Detects the current context to provide relevant command suggestions.

use crate::error::Result;
use chrono::{Datelike, Timelike};
use std::env;
use std::path::Path;

/// Current context information
#[derive(Debug, Clone)]
pub struct Context {
    pub working_directory: String,
    pub time_of_day: TimeOfDay,
    pub day_of_week: DayOfWeek,
    pub git_branch: Option<String>,
    pub project_type: Option<ProjectType>,
}

/// Time of day categories
#[derive(Debug, Clone, PartialEq)]
pub enum TimeOfDay {
    Morning,   // 6am - 12pm
    Afternoon, // 12pm - 6pm
    Evening,   // 6pm - 10pm
    Night,     // 10pm - 6am
}

/// Day of week
#[derive(Debug, Clone, PartialEq)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// Project type detected from files
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    Node,   // package.json
    Rust,   // Cargo.toml
    Python, // requirements.txt, setup.py
    Go,     // go.mod
    Java,   // pom.xml
    Ruby,   // Gemfile
    Other,
}

/// Context detector
pub struct ContextDetector;

impl ContextDetector {
    /// Detect current context
    pub fn detect() -> Result<Context> {
        let working_directory = env::current_dir()?
            .to_str()
            .unwrap_or("/")
            .to_string();

        let time_of_day = Self::detect_time_of_day();
        let day_of_week = Self::detect_day_of_week();
        let git_branch = Self::detect_git_branch();
        let project_type = Self::detect_project_type(&working_directory);

        Ok(Context {
            working_directory,
            time_of_day,
            day_of_week,
            git_branch,
            project_type,
        })
    }

    /// Detect time of day
    fn detect_time_of_day() -> TimeOfDay {
        let now = chrono::Local::now();
        let hour = now.hour();

        match hour {
            6..=11 => TimeOfDay::Morning,
            12..=17 => TimeOfDay::Afternoon,
            18..=21 => TimeOfDay::Evening,
            _ => TimeOfDay::Night,
        }
    }

    /// Detect day of week
    fn detect_day_of_week() -> DayOfWeek {
        let now = chrono::Local::now();
        match now.weekday() {
            chrono::Weekday::Mon => DayOfWeek::Monday,
            chrono::Weekday::Tue => DayOfWeek::Tuesday,
            chrono::Weekday::Wed => DayOfWeek::Wednesday,
            chrono::Weekday::Thu => DayOfWeek::Thursday,
            chrono::Weekday::Fri => DayOfWeek::Friday,
            chrono::Weekday::Sat => DayOfWeek::Saturday,
            chrono::Weekday::Sun => DayOfWeek::Sunday,
        }
    }

    /// Detect current git branch
    fn detect_git_branch() -> Option<String> {
        use std::process::Command;

        Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                } else {
                    None
                }
            })
            .map(|s| s.trim().to_string())
    }

    /// Detect project type from marker files
    fn detect_project_type(dir: &str) -> Option<ProjectType> {
        let path = Path::new(dir);

        if path.join("package.json").exists() {
            Some(ProjectType::Node)
        } else if path.join("Cargo.toml").exists() {
            Some(ProjectType::Rust)
        } else if path.join("requirements.txt").exists() || path.join("setup.py").exists() {
            Some(ProjectType::Python)
        } else if path.join("go.mod").exists() {
            Some(ProjectType::Go)
        } else if path.join("pom.xml").exists() {
            Some(ProjectType::Java)
        } else if path.join("Gemfile").exists() {
            Some(ProjectType::Ruby)
        } else {
            Some(ProjectType::Other)
        }
    }
}

impl std::fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeOfDay::Morning => write!(f, "morning"),
            TimeOfDay::Afternoon => write!(f, "afternoon"),
            TimeOfDay::Evening => write!(f, "evening"),
            TimeOfDay::Night => write!(f, "night"),
        }
    }
}

impl std::fmt::Display for DayOfWeek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DayOfWeek::Monday => write!(f, "Monday"),
            DayOfWeek::Tuesday => write!(f, "Tuesday"),
            DayOfWeek::Wednesday => write!(f, "Wednesday"),
            DayOfWeek::Thursday => write!(f, "Thursday"),
            DayOfWeek::Friday => write!(f, "Friday"),
            DayOfWeek::Saturday => write!(f, "Saturday"),
            DayOfWeek::Sunday => write!(f, "Sunday"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_context() {
        let context = ContextDetector::detect();
        assert!(context.is_ok());

        let ctx = context.unwrap();
        assert!(!ctx.working_directory.is_empty());
    }

    #[test]
    fn test_time_of_day() {
        let time = ContextDetector::detect_time_of_day();
        // Just ensure it returns something valid
        assert!(matches!(
            time,
            TimeOfDay::Morning
                | TimeOfDay::Afternoon
                | TimeOfDay::Evening
                | TimeOfDay::Night
        ));
    }

    #[test]
    fn test_day_of_week() {
        let day = ContextDetector::detect_day_of_week();
        // Just ensure it returns something valid
        assert!(matches!(
            day,
            DayOfWeek::Monday
                | DayOfWeek::Tuesday
                | DayOfWeek::Wednesday
                | DayOfWeek::Thursday
                | DayOfWeek::Friday
                | DayOfWeek::Saturday
                | DayOfWeek::Sunday
        ));
    }
}
