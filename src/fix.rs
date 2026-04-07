//! Auto-correct detected permission issues.

use anyhow::{Context, Result};

use crate::checks::Issue;

/// Apply the recommended `fix_mode` for every issue. Returns the number
/// of issues successfully fixed.
pub fn apply(issues: &[Issue]) -> Result<usize> {
    let mut fixed = 0;
    for issue in issues {
        if apply_one(issue)? {
            fixed += 1;
        }
    }
    Ok(fixed)
}

fn apply_one(issue: &Issue) -> Result<bool> {
    let Some(target) = issue.fix_mode else {
        return Ok(false);
    };

    #[cfg(unix)]
    {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let perms = fs::Permissions::from_mode(target);
        fs::set_permissions(&issue.path, perms)
            .with_context(|| format!("chmod {}", issue.path.display()))?;
        Ok(true)
    }
    #[cfg(not(unix))]
    {
        let _ = target;
        Ok(false)
    }
}
