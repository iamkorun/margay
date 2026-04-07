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
    assert!(v["issues"].as_array().unwrap().len() == 3);
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
