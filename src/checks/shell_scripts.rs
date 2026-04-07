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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn recognizes_all_shell_extensions() {
        for ext in ["sh", "bash", "zsh", "fish"] {
            let p = PathBuf::from(format!("script.{ext}"));
            // This only tests the extension match — file_mode returns None
            // because the file doesn't exist, so check() returns None.
            // We assert the extension is in the recognized list.
            let lower = ext_lower(&p).unwrap();
            assert!(SHELL_EXTS.contains(&lower.as_str()));
        }
    }

    #[test]
    fn ignores_non_shell_extensions() {
        for ext in ["rs", "py", "txt", "md", ""] {
            let name = if ext.is_empty() {
                "script".to_string()
            } else {
                format!("file.{ext}")
            };
            let p = PathBuf::from(name);
            assert!(check(&p).is_none());
        }
    }
}
