/// Shell integration module
///
/// Handles shell detection and hook installation for automatic command recording.

pub mod hook_installer;
pub mod shell_detector;

pub use hook_installer::HookInstaller;
pub use shell_detector::{Shell, ShellDetector};
