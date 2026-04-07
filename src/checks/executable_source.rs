//! Flag source files that are marked user-executable.

use std::path::Path;

use super::{ext_lower, file_mode, Category, Issue};

const SOURCE_EXTS: &[&str] = &[
    "rs", "go", "py", "ts", "js", "tsx", "jsx", "json", "toml", "yaml", "yml", "md", "txt", "html",
    "css",
];

pub fn check(path: &Path) -> Option<Issue> {
    let ext = ext_lower(path)?;
    if !SOURCE_EXTS.contains(&ext.as_str()) {
        return None;
    }
    let mode = file_mode(path)?;
    if mode & 0o111 != 0 {
        let fix_mode = mode & !0o111 & 0o7777;
        return Some(Issue {
            path: path.to_path_buf(),
            category: Category::SourceExecutable,
            mode: Some(mode),
            fix_mode: Some(fix_mode),
            message: format!("source file is marked executable (mode {:o})", mode),
        });
    }
    None
}
