//! Output formatters: colored text + machine-readable JSON.

use std::collections::BTreeMap;

use anyhow::Result;
use colored::Colorize;
use serde::Serialize;

use crate::checks::{Category, Issue};

#[derive(Serialize)]
struct JsonReport<'a> {
    total: usize,
    issues: &'a [Issue],
}

pub fn print_json(issues: &[Issue]) -> Result<()> {
    let report = JsonReport {
        total: issues.len(),
        issues,
    };
    let s = serde_json::to_string_pretty(&report)?;
    println!("{s}");
    Ok(())
}

pub fn print_text(issues: &[Issue], quiet: bool) {
    if issues.is_empty() {
        if !quiet {
            println!("{} no permission issues found", "✓".green().bold());
        }
        return;
    }

    // Group by category, preserving a stable ordering.
    let mut grouped: BTreeMap<&'static str, Vec<&Issue>> = BTreeMap::new();
    for issue in issues {
        grouped.entry(issue.category.title()).or_default().push(issue);
    }

    for (title, items) in &grouped {
        println!("{}", title.bold().underline());
        for issue in items {
            let mode_str = match issue.mode {
                Some(m) => format!("{:04o}", m),
                None => "----".into(),
            };
            let fix_str = match issue.fix_mode {
                Some(m) => format!("→ {:04o}", m),
                None => String::new(),
            };
            let marker = marker_for(issue.category);
            println!(
                "  {} {} {} {}",
                marker,
                issue.path.display().to_string().cyan(),
                mode_str.yellow(),
                fix_str.dimmed(),
            );
        }
        println!();
    }

    let file_count = {
        let mut set = std::collections::HashSet::new();
        for i in issues {
            set.insert(&i.path);
        }
        set.len()
    };
    println!(
        "{} {} issue{} found across {} file{}",
        "✗".red().bold(),
        issues.len(),
        plural(issues.len()),
        file_count,
        plural(file_count),
    );
}

fn marker_for(cat: Category) -> colored::ColoredString {
    match cat {
        Category::ShellNeedsExec => "+x".green().bold(),
        Category::SensitiveTooOpen => "!!".red().bold(),
        Category::SourceExecutable => "-x".yellow().bold(),
    }
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}
