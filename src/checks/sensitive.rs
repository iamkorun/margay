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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn classifies_dot_env_as_sensitive() {
        assert!(is_sensitive(&PathBuf::from(".env")));
        assert!(is_sensitive(&PathBuf::from(".env.production")));
        assert!(is_sensitive(&PathBuf::from(".env.local")));
        assert!(is_sensitive(&PathBuf::from("project/.env")));
    }

    #[test]
    fn classifies_key_files_as_sensitive() {
        assert!(is_sensitive(&PathBuf::from("server.key")));
        assert!(is_sensitive(&PathBuf::from("cert.pem")));
        assert!(is_sensitive(&PathBuf::from("ca.crt")));
        assert!(is_sensitive(&PathBuf::from("bundle.pfx")));
    }

    #[test]
    fn classifies_ssh_private_keys_as_sensitive() {
        assert!(is_sensitive(&PathBuf::from("id_rsa")));
        assert!(is_sensitive(&PathBuf::from("id_ed25519")));
        assert!(is_sensitive(&PathBuf::from("id_dsa")));
        assert!(is_sensitive(&PathBuf::from("id_ecdsa")));
    }

    #[test]
    fn classifies_normal_files_as_not_sensitive() {
        assert!(!is_sensitive(&PathBuf::from("README.md")));
        assert!(!is_sensitive(&PathBuf::from("main.rs")));
        assert!(!is_sensitive(&PathBuf::from("notes.txt")));
        assert!(!is_sensitive(&PathBuf::from("envrc"))); // not .env
    }

    #[test]
    fn case_insensitive_match() {
        assert!(is_sensitive(&PathBuf::from("ID_RSA")));
        assert!(is_sensitive(&PathBuf::from("server.KEY")));
    }
}
