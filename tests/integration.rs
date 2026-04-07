//! End-to-end integration tests for the `margay` CLI.

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn chmod(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

fn write(dir: &Path, name: &str, contents: &str, mode: u32) {
    let p = dir.join(name);
    fs::write(&p, contents).unwrap();
    chmod(&p, mode);
}

/// Build a fixture tree containing one issue of each category plus a
/// file that should be ignored.
fn fixture() -> TempDir {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // shell script missing +x
    write(root, "deploy.sh", "#!/bin/bash\necho hi\n", 0o644);
    // sensitive file, world-readable
    write(root, ".env", "SECRET=1\n", 0o644);
    // source file marked executable
    write(root, "lib.rs", "fn main() {}\n", 0o755);
    // clean file — should not be reported
    write(root, "README.md", "ok\n", 0o644);

    // .ignore (honored by the `ignore` crate without a git repo) should hide this one
    fs::write(root.join(".ignore"), "ignored.sh\n").unwrap();
    write(root, "ignored.sh", "#!/bin/bash\n", 0o644);

    tmp
}

#[test]
fn reports_issues_and_exits_nonzero() {
    let tmp = fixture();
    Command::cargo_bin("margay")
        .unwrap()
        .arg(tmp.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("Shell scripts missing +x"))
        .stdout(predicate::str::contains("Sensitive files too open"))
        .stdout(predicate::str::contains("Source files marked executable"))
        .stdout(predicate::str::contains("3 issues found"));
}

#[test]
fn json_mode_outputs_structured_report() {
    let tmp = fixture();
    let output = Command::cargo_bin("margay")
        .unwrap()
        .arg(tmp.path())
        .arg("--json")
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(v["total"], 3);
    let issues = v["issues"].as_array().unwrap();
    assert_eq!(issues.len(), 3);

    // Mode and fix_mode must be human-readable octal strings, not raw decimals.
    for issue in issues {
        let mode = issue["mode"].as_str().expect("mode should be a string");
        assert!(
            mode.starts_with('0') && mode.len() == 4,
            "mode should be a 4-digit octal string, got: {mode}"
        );
        let fix = issue["fix_mode"]
            .as_str()
            .expect("fix_mode should be a string");
        assert!(
            fix.starts_with('0') && fix.len() == 4,
            "fix_mode should be a 4-digit octal string, got: {fix}"
        );
    }
}

#[test]
fn fix_mode_corrects_permissions_and_exits_zero() {
    let tmp = fixture();
    let root = tmp.path();

    Command::cargo_bin("margay")
        .unwrap()
        .arg(root)
        .arg("--fix")
        .assert()
        .success();

    let shell_mode = fs::metadata(root.join("deploy.sh"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(shell_mode & 0o100, 0o100, "shell script now executable");

    let env_mode = fs::metadata(root.join(".env"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(env_mode & 0o077, 0, ".env no longer group/world readable");

    let src_mode = fs::metadata(root.join("lib.rs"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(src_mode & 0o111, 0, "source file no longer executable");

    // Second run should find nothing.
    Command::cargo_bin("margay")
        .unwrap()
        .arg(root)
        .assert()
        .success();
}

#[test]
fn clean_tree_exits_zero() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "README.md", "hi\n", 0o644);
    Command::cargo_bin("margay")
        .unwrap()
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("no permission issues"));
}

#[test]
fn missing_path_is_friendly_error() {
    Command::cargo_bin("margay")
        .unwrap()
        .arg("/this/does/not/exist/margay-xyz")
        .assert()
        .failure()
        .stderr(predicate::str::contains("path does not exist"));
}

#[test]
fn json_and_fix_are_mutually_exclusive() {
    Command::cargo_bin("margay")
        .unwrap()
        .arg("--json")
        .arg("--fix")
        .assert()
        .failure();
}

#[test]
fn detects_multiple_sensitive_file_kinds() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write(root, ".env.production", "X=1\n", 0o644);
    write(root, "server.key", "key\n", 0o644);
    write(root, "id_rsa", "ssh\n", 0o660);

    Command::cargo_bin("margay")
        .unwrap()
        .arg(root)
        .assert()
        .failure()
        .stdout(predicate::str::contains(".env.production"))
        .stdout(predicate::str::contains("server.key"))
        .stdout(predicate::str::contains("id_rsa"))
        .stdout(predicate::str::contains("3 issues found"));
}

#[test]
fn nested_directories_are_walked() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let nested = root.join("a/b/c");
    fs::create_dir_all(&nested).unwrap();
    write(&nested, "deploy.sh", "#!/bin/bash\n", 0o644);

    Command::cargo_bin("margay")
        .unwrap()
        .arg(root)
        .assert()
        .failure()
        .stdout(predicate::str::contains("deploy.sh"));
}

#[test]
fn gitignore_is_respected() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    // initialise a git repo so .gitignore is honored
    Command::new("git")
        .arg("init")
        .arg("--quiet")
        .current_dir(root)
        .assert()
        .success();
    fs::write(root.join(".gitignore"), "ignored.sh\n").unwrap();
    write(root, "ignored.sh", "#!/bin/bash\n", 0o644);
    write(root, "watched.sh", "#!/bin/bash\n", 0o644);

    Command::cargo_bin("margay")
        .unwrap()
        .arg(root)
        .assert()
        .failure()
        .stdout(predicate::str::contains("watched.sh"))
        .stdout(predicate::str::contains("watched.sh").count(1))
        .stdout(predicate::str::contains("1 issue"));
}

#[test]
fn single_file_path_is_accepted() {
    let tmp = TempDir::new().unwrap();
    let p = tmp.path().join("deploy.sh");
    fs::write(&p, "#!/bin/bash\n").unwrap();
    chmod(&p, 0o644);

    Command::cargo_bin("margay")
        .unwrap()
        .arg(&p)
        .assert()
        .failure()
        .stdout(predicate::str::contains("deploy.sh"));
}

#[test]
fn quiet_mode_suppresses_clean_message() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "README.md", "ok\n", 0o644);

    let output = Command::cargo_bin("margay")
        .unwrap()
        .arg(tmp.path())
        .arg("--quiet")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(
        output.is_empty(),
        "expected empty stdout in --quiet mode on a clean tree, got: {}",
        String::from_utf8_lossy(&output)
    );
}

#[test]
fn quiet_mode_collapses_issue_listing_to_summary() {
    let tmp = fixture();
    let output = Command::cargo_bin("margay")
        .unwrap()
        .arg(tmp.path())
        .arg("--quiet")
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8_lossy(&output);
    // Summary line is present
    assert!(s.contains("3 issues found"), "summary missing: {s}");
    // Per-category listings are NOT present
    assert!(
        !s.contains("Shell scripts missing +x"),
        "category header should be hidden in --quiet: {s}"
    );
    assert!(
        !s.contains("Sensitive files too open"),
        "category header should be hidden in --quiet: {s}"
    );
}

#[test]
fn version_flag_prints_version() {
    Command::cargo_bin("margay")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("margay"));
}
