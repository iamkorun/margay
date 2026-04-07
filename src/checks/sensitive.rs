//! Flag sensitive files with permissions looser than 0o600.

use std::path::Path;

use super::{ext_lower, file_mode, name_lower, Category, Issue};

const SENSITIVE_EXTS: &[&str] = &["key", "pem", "crt", "pfx"];
const SENSITIVE_NAMES: &[&str] = &["id_rsa", "id_ed25519", "id_dsa", "id_ecdsa"];

fn is_sensitive(path: &Path) -> bool {
    if let Some(name) = name_lower(path) {
        if SENSITIVE_NAMES.contains(&name.as_str()) {
            return true;
        }
        if name == ".env" || name.starts_with(".env.") {
            return true;
        }
    }
    if let Some(ext) = ext_lower(path) {
        if SENSITIVE_EXTS.contains(&ext.as_str()) {
            return true;
        }
    }
    false
}

pub fn check(path: &Path) -> Option<Issue> {
    if !is_sensitive(path) {
        return None;
    }
    let mode = file_mode(path)?;
    // Anything in the group/other bits is "too open" for a secret.
    if mode & 0o077 != 0 {
        // The fix is always 0o600 (owner read+write, nothing else):
        // sensitive files don't need to be executable, and they
        // certainly don't need to be group/world readable.
        return Some(Issue {
            path: path.to_path_buf(),
            category: Category::SensitiveTooOpen,
            mode: Some(mode),
            fix_mode: Some(0o600),
            message: format!("sensitive file is group/world accessible (mode {mode:o})"),
        });
    }
    None
}
