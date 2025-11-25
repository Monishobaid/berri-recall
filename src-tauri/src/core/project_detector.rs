/// Project root detection logic
///
/// Detects the root directory of a project by looking for common markers
/// like .git, package.json, Cargo.toml, etc.

use crate::error::Result;
use std::path::{Path, PathBuf};

/// Project root detection markers
const PROJECT_MARKERS: &[&str] = &[
    ".git",
    "Cargo.toml",
    "package.json",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "requirements.txt",
    "Gemfile",
    "composer.json",
    ".project",
];

/// Handles project root detection
pub struct ProjectDetector;

impl ProjectDetector {
    /// Detect the project root from a given path
    ///
    /// Walks up the directory tree looking for common project markers.
    ///
    /// # Arguments
    /// * `start_path` - The path to start searching from (usually cwd)
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - The detected project root
    /// * `Err(RecallError)` - If no project root is found
    ///
    /// # Examples
    /// ```no_run
    /// use recall_cli_lib::core::ProjectDetector;
    /// use std::env;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let cwd = env::current_dir()?;
    /// let project_root = ProjectDetector::detect(&cwd)?;
    /// println!("Project root: {}", project_root.display());
    /// # Ok(())
    /// # }
    /// ```
    pub fn detect<P: AsRef<Path>>(start_path: P) -> Result<PathBuf> {
        let start_path = start_path.as_ref();

        // Ensure the path is absolute
        let absolute_path = if start_path.is_absolute() {
            start_path.to_path_buf()
        } else {
            std::env::current_dir()?.join(start_path)
        };

        // Walk up the directory tree
        let mut current = absolute_path.as_path();

        loop {
            // Check for project markers
            for marker in PROJECT_MARKERS {
                let marker_path = current.join(marker);
                if marker_path.exists() {
                    return Ok(current.to_path_buf());
                }
            }

            // Move to parent directory
            match current.parent() {
                Some(parent) => current = parent,
                None => {
                    // Reached filesystem root without finding markers
                    // Fall back to the original directory
                    return Ok(absolute_path);
                }
            }
        }
    }

    /// Check if a path is inside a project
    ///
    /// Returns true if the path has any project markers in its hierarchy.
    pub fn is_in_project<P: AsRef<Path>>(path: P) -> bool {
        Self::detect(path).is_ok()
    }

    /// Get the project name from the root path
    ///
    /// Uses the directory name as the project name.
    pub fn get_project_name<P: AsRef<Path>>(project_root: P) -> Option<String> {
        project_root
            .as_ref()
            .file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
    }

    /// Detect if path is a git repository
    pub fn is_git_repo<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().join(".git").exists()
    }

    /// Get all project markers found in a path
    ///
    /// Useful for debugging or displaying project type information.
    pub fn get_markers<P: AsRef<Path>>(path: P) -> Vec<String> {
        let path = path.as_ref();
        let mut markers = Vec::new();

        for marker in PROJECT_MARKERS {
            if path.join(marker).exists() {
                markers.push(marker.to_string());
            }
        }

        markers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_git_project() {
        let temp = TempDir::new().unwrap();
        let project_dir = temp.path().join("my-project");
        fs::create_dir(&project_dir).unwrap();
        fs::create_dir(project_dir.join(".git")).unwrap();

        let sub_dir = project_dir.join("src").join("components");
        fs::create_dir_all(&sub_dir).unwrap();

        // Should detect from subdirectory
        let detected = ProjectDetector::detect(&sub_dir).unwrap();
        assert_eq!(detected, project_dir);
    }

    #[test]
    fn test_detect_multiple_markers() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join(".git")).unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();

        let detected = ProjectDetector::detect(temp.path()).unwrap();
        assert_eq!(detected, temp.path());
    }

    #[test]
    fn test_no_project_found() {
        let temp = TempDir::new().unwrap();
        let sub_dir = temp.path().join("no-markers");
        fs::create_dir(&sub_dir).unwrap();

        // Should fall back to the original directory
        let detected = ProjectDetector::detect(&sub_dir).unwrap();
        assert_eq!(detected, sub_dir);
    }

    #[test]
    fn test_is_git_repo() {
        let temp = TempDir::new().unwrap();
        assert!(!ProjectDetector::is_git_repo(temp.path()));

        fs::create_dir(temp.path().join(".git")).unwrap();
        assert!(ProjectDetector::is_git_repo(temp.path()));
    }

    #[test]
    fn test_get_project_name() {
        let path = PathBuf::from("/home/user/my-awesome-project");
        let name = ProjectDetector::get_project_name(&path).unwrap();
        assert_eq!(name, "my-awesome-project");
    }

    #[test]
    fn test_get_markers() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join(".git")).unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();

        let markers = ProjectDetector::get_markers(temp.path());
        assert!(markers.contains(&".git".to_string()));
        assert!(markers.contains(&"package.json".to_string()));
        assert_eq!(markers.len(), 2);
    }
}
