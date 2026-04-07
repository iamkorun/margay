//! Flag shell scripts that are missing the user-execute bit.

use std::path::Path;

use super::{ext_lower, file_mode, Category, Issue};

const SHELL_EXTS: &[&str] = &["sh", "bash", "zsh", "fish"];

pub fn check(path: &Path) -> Option<Issue> {
    let ext = ext_lower(path)?;
    if !SHELL_EXTS.contains(&ext.as_str()) {
        return None;
    }
    let mode = file_mode(path)?;
    if mode & 0o100 == 0 {
        let fix_mode = mode | 0o100;
        return Some(Issue {
            path: path.to_path_buf(),
            category: Category::ShellNeedsExec,
            mode: Some(mode),
            fix_mode: Some(fix_mode),
            message: format!("shell script is not user-executable (mode {mode:o})"),
        });
    }
    None
}
