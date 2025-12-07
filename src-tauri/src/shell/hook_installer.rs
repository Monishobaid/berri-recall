/// Hook installer
///
/// Handles installation and uninstallation of shell hooks.

use crate::error::{RecallError, Result};
use crate::shell::{Shell, ShellDetector};
use std::fs;
use std::path::PathBuf;

/// Hook file contents embedded at compile time
const BASH_HOOK: &str = include_str!("../../../hooks/bash.sh");
const ZSH_HOOK: &str = include_str!("../../../hooks/zsh.sh");
const FISH_HOOK: &str = include_str!("../../../hooks/fish.fish");
const POWERSHELL_HOOK: &str = include_str!("../../../hooks/powershell.ps1");

/// Hook installer
pub struct HookInstaller {
    hooks_dir: PathBuf,
}

impl HookInstaller {
    /// Create a new hook installer
    ///
    /// # Returns
    /// * `Ok(HookInstaller)` - New installer instance
    /// * `Err(RecallError)` - If home directory cannot be determined
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| RecallError::Config("Could not determine home directory".to_string()))?;

        let hooks_dir = home.join(".berri-recall").join("hooks");

        Ok(Self { hooks_dir })
    }

    /// Install hooks for the detected shell
    ///
    /// # Returns
    /// * `Ok(Shell)` - The shell that was configured
    /// * `Err(RecallError)` - If installation fails
    pub fn install_auto(&self) -> Result<Shell> {
        let shell = ShellDetector::detect()?;
        self.install(shell)?;
        Ok(shell)
    }

    /// Install hooks for a specific shell
    ///
    /// # Arguments
    /// * `shell` - The shell to install hooks for
    ///
    /// # Returns
    /// * `Ok(())` - Installation successful
    /// * `Err(RecallError)` - If installation fails
    pub fn install(&self, shell: Shell) -> Result<()> {
        // Create hooks directory if it doesn't exist
        fs::create_dir_all(&self.hooks_dir)?;

        // Write hook file
        let hook_path = self.hooks_dir.join(shell.hook_filename());
        let hook_content = self.get_hook_content(shell);

        fs::write(&hook_path, hook_content)?;

        // Make hook executable (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
        }

        // Add source line to RC file
        self.update_rc_file(shell, &hook_path)?;

        Ok(())
    }

    /// Install hooks for all detected shells
    ///
    /// # Returns
    /// * `Ok(Vec<Shell>)` - List of shells that were configured
    pub fn install_all(&self) -> Result<Vec<Shell>> {
        let shells = ShellDetector::detect_all();
        let mut installed = Vec::new();

        for shell in shells {
            match self.install(shell) {
                Ok(()) => installed.push(shell),
                Err(e) => {
                    eprintln!("Warning: Failed to install {} hook: {}", shell, e);
                }
            }
        }

        if installed.is_empty() {
            return Err(RecallError::Config(
                "No shells could be configured".to_string(),
            ));
        }

        Ok(installed)
    }

    /// Uninstall hooks for a specific shell
    ///
    /// # Arguments
    /// * `shell` - The shell to uninstall hooks from
    pub fn uninstall(&self, shell: Shell) -> Result<()> {
        let hook_path = self.hooks_dir.join(shell.hook_filename());
        let rc_path = shell.rc_file_path()?;

        // Remove source line from RC file
        if rc_path.exists() {
            let content = fs::read_to_string(&rc_path)?;
            let source_cmd = shell.source_command(&hook_path);

            let new_content: String = content
                .lines()
                .filter(|line| !line.contains(&source_cmd) && !line.contains("recall-cli"))
                .collect::<Vec<_>>()
                .join("\n");

            fs::write(&rc_path, new_content)?;
        }

        // Remove hook file
        if hook_path.exists() {
            fs::remove_file(&hook_path)?;
        }

        Ok(())
    }

    /// Check if hooks are installed for a shell
    ///
    /// # Arguments
    /// * `shell` - The shell to check
    ///
    /// # Returns
    /// * `true` if hooks are installed, `false` otherwise
    pub fn is_installed(&self, shell: Shell) -> bool {
        let hook_path = self.hooks_dir.join(shell.hook_filename());
        if !hook_path.exists() {
            return false;
        }

        // Check if RC file contains source line
        if let Ok(rc_path) = shell.rc_file_path() {
            if let Ok(content) = fs::read_to_string(&rc_path) {
                let source_cmd = shell.source_command(&hook_path);
                return content.contains(&source_cmd);
            }
        }

        false
    }

    /// Get hook content for a specific shell
    fn get_hook_content(&self, shell: Shell) -> &str {
        match shell {
            Shell::Bash => BASH_HOOK,
            Shell::Zsh => ZSH_HOOK,
            Shell::Fish => FISH_HOOK,
            Shell::PowerShell => POWERSHELL_HOOK,
        }
    }

    /// Update the RC file to source the hook
    fn update_rc_file(&self, shell: Shell, hook_path: &PathBuf) -> Result<()> {
        let rc_path = shell.rc_file_path()?;

        // Create parent directories if they don't exist
        if let Some(parent) = rc_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Read existing content or create new file
        let mut content = if rc_path.exists() {
            fs::read_to_string(&rc_path)?
        } else {
            String::new()
        };

        // Check if already installed
        let source_cmd = shell.source_command(hook_path);
        if content.contains(&source_cmd) {
            return Ok(()); // Already installed
        }

        // Add source line
        if !content.ends_with('\n') && !content.is_empty() {
            content.push('\n');
        }

        content.push_str("\n# berri-recall hook (auto-generated)\n");
        content.push_str(&source_cmd);
        content.push('\n');

        // Write back
        fs::write(&rc_path, content)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_installer() -> (HookInstaller, TempDir) {
        let temp = TempDir::new().unwrap();
        let installer = HookInstaller {
            hooks_dir: temp.path().join("hooks"),
        };
        (installer, temp)
    }

    #[test]
    fn test_new_installer() {
        let result = HookInstaller::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_hook_content() {
        let (installer, _temp) = create_test_installer();

        let bash_content = installer.get_hook_content(Shell::Bash);
        assert!(bash_content.contains("bash"));
        assert!(bash_content.contains("__recall_hook"));

        let zsh_content = installer.get_hook_content(Shell::Zsh);
        assert!(zsh_content.contains("zsh"));
    }

    #[test]
    fn test_is_installed() {
        let (installer, _temp) = create_test_installer();

        // Should not be installed initially
        assert!(!installer.is_installed(Shell::Bash));
    }
}
