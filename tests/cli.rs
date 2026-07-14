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
        .stdout(predicate::str::contains("DPCS-COM-005"));
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
fn validate_parse_failure_json_includes_parse_stage() {
    let mut cmd = bin();
    cmd.args(["validate", "does-not-exist.dpcs.yaml", "--json"]);
    cmd.assert().failure().code(2);

    let mut malformed = bin();
    malformed.args([
        "validate",
        &format!(
            "{}/tests/fixtures/invalid/malformed.dpcs.yaml",
            env!("CARGO_MANIFEST_DIR")
        ),
        "--json",
    ]);
    malformed
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::contains("\"stage\": \"parse\""))
        .stdout(predicate::str::contains("DPCS-PARSE-001"));
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
fn inspect_parse_failure_json_includes_parse_stage() {
    bin()
        .args(["inspect", &fixture("invalid/malformed.dpcs.yaml"), "--json"])
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::contains("\"stage\": \"parse\""))
        .stdout(predicate::str::contains("DPCS-PARSE-"));
}

#[test]
fn graph_parse_failure_json_includes_parse_stage() {
    bin()
        .args(["graph", &fixture("invalid/malformed.dpcs.yaml"), "--json"])
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::contains("\"stage\": \"parse\""));
}

#[test]
fn graph_json_includes_entry_and_exit_points() {
    bin()
        .args([
            "graph",
            &fixture("valid/with_graph_features.dpcs.yaml"),
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"entryPoints\""))
        .stdout(predicate::str::contains("\"exitPoints\""))
        .stdout(predicate::str::contains("\"stepOrder\""));
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
        .stdout(predicate::str::contains("DPCS-COM-005"));
}

#[test]
fn graph_json_omits_step_order_when_planning_refused() {
    bin()
        .args(["graph", &fixture("invalid/cycle.dpcs.yaml"), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"planningRefused\": true"))
        .stdout(predicate::str::contains("\"stepOrder\"").not());
}

#[test]
fn inspect_json_signals_planning_refused() {
    bin()
        .args(["inspect", &fixture("invalid/cycle.dpcs.yaml"), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"planningRefused\": true"))
        .stdout(predicate::str::contains("\"valid\": false"));
}
