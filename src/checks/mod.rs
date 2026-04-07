//! Permission checks.

use std::path::{Path, PathBuf};

use serde::Serialize;

pub mod executable_source;
pub mod sensitive;
pub mod shell_scripts;

/// One detected permission issue.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Issue {
    pub path: PathBuf,
    pub category: Category,
    /// Current mode (only the low 12 bits are meaningful). None on Windows.
    pub mode: Option<u32>,
    /// Mode we would set in --fix mode. None on Windows.
    pub fix_mode: Option<u32>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    /// Shell script missing the user-execute bit.
    ShellNeedsExec,
    /// Sensitive file with permissions looser than 0o600.
    SensitiveTooOpen,
    /// Source file that should not be executable.
    SourceExecutable,
}

impl Category {
    pub fn title(self) -> &'static str {
        match self {
            Category::ShellNeedsExec => "Shell scripts missing +x",
            Category::SensitiveTooOpen => "Sensitive files too open",
            Category::SourceExecutable => "Source files marked executable",
        }
    }
}

/// Run every check against every file and collect all issues.
pub fn run_all(files: &[PathBuf]) -> Vec<Issue> {
    let mut issues = Vec::new();
    for path in files {
        if let Some(issue) = shell_scripts::check(path) {
            issues.push(issue);
        }
        if let Some(issue) = sensitive::check(path) {
            issues.push(issue);
        }
        if let Some(issue) = executable_source::check(path) {
            issues.push(issue);
        }
    }
    issues
}

/// Read the permission mode for `path`. Returns `None` on platforms
/// without Unix permissions (e.g. Windows) or if the metadata call fails.
pub fn file_mode(path: &Path) -> Option<u32> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = std::fs::symlink_metadata(path).ok()?;
        if meta.file_type().is_symlink() {
            return None;
        }
        Some(meta.permissions().mode() & 0o7777)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        None
    }
}

/// Lowercase file extension, if any.
pub fn ext_lower(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
}

/// Lowercase file name, if any.
pub fn name_lower(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_ascii_lowercase())
}
