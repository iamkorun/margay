//! margay — audit file permissions in a project tree.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

pub mod checks;
pub mod fix;
pub mod report;
pub mod walker;

/// Audit file permissions in a project tree.
///
/// margay walks your project (respecting .gitignore) and flags:
///
/// - shell scripts that are missing the user-execute bit
/// - sensitive files (.env, *.key, *.pem, id_rsa, ...) with permissions
///   looser than 0600
/// - source files (*.rs, *.py, *.md, ...) that are marked executable
///
/// Examples:
///
///   margay                    # audit current directory
///   margay ./my-project       # audit a specific path
///   margay --fix              # auto-correct issues
///   margay --json             # machine-readable output for CI
#[derive(Debug, Parser)]
#[command(name = "margay", version, about = "Audit file permissions in a project tree", long_about = None)]
pub struct Cli {
    /// Path to scan (defaults to current directory).
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Auto-correct all detected issues.
    #[arg(long)]
    pub fix: bool,

    /// Emit machine-readable JSON instead of the colored report.
    #[arg(long, conflicts_with = "fix")]
    pub json: bool,

    /// Suppress non-essential output.
    #[arg(short, long, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Print extra detail while scanning.
    #[arg(short, long)]
    pub verbose: bool,
}

/// Library entry point used by `main`.
pub fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    run_with(cli)
}

/// Test-friendly entry point.
pub fn run_with(cli: Cli) -> Result<ExitCode> {
    let root = cli.path.clone().unwrap_or_else(|| PathBuf::from("."));

    if !root.exists() {
        anyhow::bail!("path does not exist: {}", root.display());
    }

    let files = walker::collect_files(&root)?;
    if cli.verbose && !cli.json {
        eprintln!("margay: scanned {} files", files.len());
    }

    let issues = checks::run_all(&files);

    if cli.fix {
        let fixed = fix::apply(&issues)?;
        if !cli.quiet {
            println!("margay: fixed {fixed} issue(s)");
        }
        return Ok(ExitCode::SUCCESS);
    }

    if cli.json {
        report::print_json(&issues)?;
    } else {
        report::print_text(&issues, cli.quiet);
    }

    if issues.is_empty() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}
