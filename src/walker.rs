//! Directory walker that respects .gitignore.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ignore::WalkBuilder;

/// Collect every file under `root`, honoring .gitignore rules.
///
/// Symlinks are not followed. Directories are skipped (we only
/// care about regular files for the permission checks).
pub fn collect_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let walker = WalkBuilder::new(root)
        .hidden(false) // include dotfiles — .env matters!
        .follow_links(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    for dent in walker {
        let dent = match dent {
            Ok(d) => d,
            Err(e) => {
                eprintln!("margay: skipping entry: {e}");
                continue;
            }
        };

        let ft = match dent.file_type() {
            Some(ft) => ft,
            None => continue,
        };

        if !ft.is_file() {
            continue;
        }

        files.push(dent.into_path());
    }

    // Deterministic order makes output stable and tests reliable.
    files.sort();
    Ok(files)
}

/// Canonicalize a path, falling back to the original on error.
#[allow(dead_code)]
pub fn canonicalize_or(path: &Path) -> PathBuf {
    path.canonicalize()
        .with_context(|| format!("canonicalize {}", path.display()))
        .unwrap_or_else(|_| path.to_path_buf())
}
