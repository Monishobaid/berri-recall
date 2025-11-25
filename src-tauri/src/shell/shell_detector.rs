/// Shell detection logic
///
/// Detects which shell the user is running and provides shell-specific configuration paths.

use crate::error::{RecallError, Result};
use std::env;
use std::path::PathBuf;

/// Supported shells
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

impl Shell {
    /// Get the shell name as a string
    pub fn name(&self) -> &str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::PowerShell => "powershell",
        }
    }

    /// Get the hook file name for this shell
    pub fn hook_filename(&self) -> &str {
        match self {
            Shell::Bash => "bash.sh",
            Shell::Zsh => "zsh.sh",
            Shell::Fish => "fish.fish",
            Shell::PowerShell => "powershell.ps1",
        }
    }

    /// Get the RC file path for this shell
    ///
    /// Returns the configuration file that should be modified to source the hook.
    pub fn rc_file_path(&self) -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            RecallError::Config("Could not determine home directory".to_string())
        })?;

        let path = match self {
            Shell::Bash => {
                // Prefer .bashrc, fallback to .bash_profile
                let bashrc = home.join(".bashrc");
                if bashrc.exists() {
                    bashrc
                } else {
                    home.join(".bash_profile")
                }
            }
            Shell::Zsh => home.join(".zshrc"),
            Shell::Fish => home.join(".config/fish/config.fish"),
            Shell::PowerShell => {
                // PowerShell profile location
                home.join("Documents/PowerShell/Microsoft.PowerShell_profile.ps1")
            }
        };

        Ok(path)
    }

    /// Get the source command for this shell
    ///
    /// Returns the command to add to the RC file to source the hook.
    pub fn source_command(&self, hook_path: &PathBuf) -> String {
        match self {
            Shell::Bash | Shell::Zsh => {
                format!("[ -f \"{}\" ] && source \"{}\"", hook_path.display(), hook_path.display())
            }
            Shell::Fish => {
                format!("test -f \"{}\" && source \"{}\"", hook_path.display(), hook_path.display())
            }
            Shell::PowerShell => {
                format!(". \"{}\"", hook_path.display())
            }
        }
    }
}

impl std::fmt::Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Shell detector
pub struct ShellDetector;

impl ShellDetector {
    /// Detect the current shell
    ///
    /// Attempts to detect the shell from environment variables.
    ///
    /// # Returns
    /// * `Ok(Shell)` - The detected shell
    /// * `Err(RecallError)` - If shell cannot be detected
    pub fn detect() -> Result<Shell> {
        // Check SHELL environment variable
        if let Ok(shell_path) = env::var("SHELL") {
            let shell_name = shell_path
                .split('/')
                .last()
                .unwrap_or("")
                .to_lowercase();

            return match shell_name.as_str() {
                "bash" => Ok(Shell::Bash),
                "zsh" => Ok(Shell::Zsh),
                "fish" => Ok(Shell::Fish),
                _ => Err(RecallError::Config(format!(
                    "Unsupported shell: {}",
                    shell_name
                ))),
            };
        }

        // Check for PowerShell
        if env::var("PSModulePath").is_ok() {
            return Ok(Shell::PowerShell);
        }

        // Fallback: try to detect from parent process
        #[cfg(target_os = "macos")]
        if Self::is_process_running("zsh") {
            return Ok(Shell::Zsh);
        }

        #[cfg(target_os = "macos")]
        if Self::is_process_running("bash") {
            return Ok(Shell::Bash);
        }

        Err(RecallError::Config(
            "Could not detect shell. Please set $SHELL environment variable.".to_string(),
        ))
    }

    /// Detect all shells installed on the system
    ///
    /// Returns a list of shells that have RC files present.
    pub fn detect_all() -> Vec<Shell> {
        let mut shells = Vec::new();

        for shell in &[Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
            if let Ok(rc_path) = shell.rc_file_path() {
                // Check if parent directory exists (for fish, PowerShell)
                if let Some(parent) = rc_path.parent() {
                    if parent.exists() {
                        shells.push(*shell);
                    }
                } else if rc_path.exists() {
                    shells.push(*shell);
                }
            }
        }

        shells
    }

    /// Check if a process with the given name is running
    #[cfg(target_os = "macos")]
    fn is_process_running(name: &str) -> bool {
        use std::process::Command;

        Command::new("pgrep")
            .arg(name)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_name() {
        assert_eq!(Shell::Bash.name(), "bash");
        assert_eq!(Shell::Zsh.name(), "zsh");
        assert_eq!(Shell::Fish.name(), "fish");
        assert_eq!(Shell::PowerShell.name(), "powershell");
    }

    #[test]
    fn test_hook_filename() {
        assert_eq!(Shell::Bash.hook_filename(), "bash.sh");
        assert_eq!(Shell::Zsh.hook_filename(), "zsh.sh");
        assert_eq!(Shell::Fish.hook_filename(), "fish.fish");
        assert_eq!(Shell::PowerShell.hook_filename(), "powershell.ps1");
    }

    #[test]
    fn test_shell_display() {
        assert_eq!(Shell::Bash.to_string(), "bash");
        assert_eq!(Shell::Zsh.to_string(), "zsh");
    }

    #[test]
    fn test_rc_file_path() {
        // Should not panic
        let _ = Shell::Bash.rc_file_path();
        let _ = Shell::Zsh.rc_file_path();
        let _ = Shell::Fish.rc_file_path();
    }

    #[test]
    fn test_source_command() {
        let path = PathBuf::from("/home/user/.recall/bash.sh");

        let bash_cmd = Shell::Bash.source_command(&path);
        assert!(bash_cmd.contains("source"));
        assert!(bash_cmd.contains("/home/user/.recall/bash.sh"));

        let fish_cmd = Shell::Fish.source_command(&path);
        assert!(fish_cmd.contains("source"));
    }

    #[test]
    fn test_detect_all() {
        let shells = ShellDetector::detect_all();
        // Should return at least one shell
        assert!(!shells.is_empty());
    }
}
