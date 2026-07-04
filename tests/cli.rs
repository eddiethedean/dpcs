//! CLI integration tests.

use assert_cmd::Command;
use predicates::prelude::*;

fn bin() -> Command {
    Command::cargo_bin("dpcs").unwrap()
}

fn fixture(path: &str) -> String {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(path)
        .display()
        .to_string()
}

#[test]
fn version_succeeds() {
    bin()
        .arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("dpcs"));
}

#[test]
fn validate_success() {
    bin()
        .args(["validate", &fixture("valid/minimal.dpcs.yaml")])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn validate_failure_for_duplicates() {
    bin()
        .args(["validate", &fixture("invalid/duplicate_steps.dpcs.yaml")])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("DPCS-STR-002"));
}

#[test]
fn validate_parse_failure() {
    bin()
        .args(["validate", "does-not-exist.dpcs.yaml"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn inspect_json() {
    bin()
        .args(["inspect", &fixture("valid/minimal.dpcs.yaml"), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\": \"valid.minimal\""));
}

#[test]
fn diagnostics_json() {
    bin()
        .args([
            "diagnostics",
            &fixture("invalid/duplicate_steps.dpcs.yaml"),
            "--json",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("DPCS-STR-002"));
}
