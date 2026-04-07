//! margay — audit file permissions in a project tree.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

pub mod checks;
pub mod fix;
pub mod report;
pub mod walker;

#[derive(Debug, Parser)]
#[command(
    name = "margay",
    version,
    about = "Audit file permissions in a project tree",
    long_about = "Audit file permissions in a project tree.\n\n\
        margay walks your project (respecting .gitignore) and flags three \
        kinds of trouble:\n\n  \
        * shell scripts that are missing the user-execute bit\n  \
        * sensitive files (.env, *.key, *.pem, id_rsa, ...) with permissions \
        looser than 0600\n  \
        * source files (*.rs, *.py, *.md, ...) that are marked executable",
    after_help = "EXAMPLES:\n  \
        margay                    audit the current directory\n  \
        margay ./my-project       audit a specific path\n  \
        margay --fix              auto-correct every detected issue\n  \
        margay --json             machine-readable output for CI\n  \
        margay --quiet            silence the clean-tree banner\n\n\
        EXIT CODES:\n  \
        0  no issues (or --fix succeeded)\n  \
        1  issues were reported\n  \
        2  bad invocation (path missing, etc.)\n\n\
        Issue tracker: https://github.com/iamkorun/margay/issues"
)]
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
    let root = cli.path.unwrap_or_else(|| PathBuf::from("."));

    if !root.exists() {
        anyhow::bail!("path does not exist: {}", root.display());
    }

    if !root.is_dir() && !root.is_file() {
        anyhow::bail!(
            "path is not a regular file or directory: {}",
            root.display()
        );
    }

    let files = walker::collect_files(&root, cli.verbose)?;
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
