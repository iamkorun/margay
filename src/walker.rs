//! Directory walker that respects .gitignore.

use std::path::{Path, PathBuf};

use anyhow::Result;
use ignore::WalkBuilder;

/// Collect every file under `root`, honoring .gitignore rules.
///
/// Symlinks are not followed. Directories are skipped (we only
/// care about regular files for the permission checks).
///
/// When `verbose` is true, walker errors (e.g. permission denied
/// on a subdirectory) are reported on stderr; otherwise they are
/// silently skipped to keep CI output clean.
pub fn collect_files(root: &Path, verbose: bool) -> Result<Vec<PathBuf>> {
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
                if verbose {
                    eprintln!("margay: skipping entry: {e}");
                }
                continue;
            }
        };

        let Some(ft) = dent.file_type() else {
            continue;
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
