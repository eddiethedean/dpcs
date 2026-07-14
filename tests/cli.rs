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
        .stdout(predicate::str::contains("DPCS-COM-005"))
        .stdout(predicate::str::contains("processingResult"))
        .stdout(predicate::str::contains("implementation"));
}

#[test]
fn validate_strict_fails_on_warnings() {
    // DOC-002 is a warning for unsupported major versions.
    let yaml = format!("{}/strict_doc_warn.yaml", std::env::temp_dir().display());
    std::fs::write(
        &yaml,
        r#"
dpcsVersion: "9.0.0"
id: "warn.pipeline"
version: "0.1.0"
interface:
  inputs: []
  outputs: []
steps:
  - id: "a"
    type: "extension:noop"
graph:
  edges: []
"#,
    )
    .unwrap();
    bin()
        .args(["validate", &yaml])
        .assert()
        .success()
        .stdout(predicate::str::contains("DPCS-DOC-002"));
    bin()
        .args(["validate", &yaml, "--strict"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn validate_with_conformance_profile_requires_security() {
    let profile = format!(
        "{}/require_security.profile.yaml",
        std::env::temp_dir().display()
    );
    std::fs::write(
        &profile,
        r#"
id: "require.security"
version: "0.1.0"
dpcsVersion: "1.0.0-draft"
levels: [validator]
requireSecurity: true
"#,
    )
    .unwrap();
    bin()
        .args([
            "validate",
            &fixture("valid/minimal.dpcs.yaml"),
            "--profile",
            &profile,
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("DPCS-CONF-012"));
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

#[test]
fn capabilities_match_example_pair() {
    let profile = format!(
        "{}/examples/orchestrator.capabilities.yaml",
        env!("CARGO_MANIFEST_DIR")
    );
    let contract = format!(
        "{}/examples/with_execution.dpcs.yaml",
        env!("CARGO_MANIFEST_DIR")
    );
    bin()
        .args(["capabilities", &profile, "--plan", &contract])
        .assert()
        .success()
        .stdout(predicate::str::contains("match: ok"));
}

#[test]
fn capabilities_json_reports_missing_mandatory() {
    bin()
        .args([
            "capabilities",
            &fixture("capabilities/invalid/missing_mandatory.profile.yaml"),
            "--plan",
            &fixture("valid/with_execution_model.dpcs.yaml"),
            "--json",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("DPCS-CAP-005"))
        .stdout(predicate::str::contains("\"missingMandatory\""))
        .stdout(predicate::str::contains("sql.readwrite"));
}

#[test]
fn bind_success_writes_files() {
    let out = tempfile::tempdir().unwrap();
    bin()
        .args([
            "bind",
            &fixture("valid/with_execution_model.dpcs.yaml"),
            "--profile",
            &fixture("capabilities/valid/matching.profile.yaml"),
            "--target",
            "airflow",
            "--out",
            out.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("bind: ok"))
        .stdout(predicate::str::contains("target: airflow"));

    let entries: Vec<_> = std::fs::read_dir(out.path().join("dags"))
        .unwrap()
        .collect();
    assert!(!entries.is_empty());
}

#[test]
fn bind_json_emits_bundle() {
    let out = tempfile::tempdir().unwrap();
    bin()
        .args([
            "bind",
            &fixture("valid/with_execution_model.dpcs.yaml"),
            "--profile",
            &fixture("capabilities/valid/matching.profile.yaml"),
            "--target",
            "prefect",
            "--out",
            out.path().to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"target\": \"prefect\""))
        .stdout(predicate::str::contains("\"contractId\""))
        .stdout(predicate::str::contains("\"files\""));
}

#[test]
fn bind_capability_failure_exits_one() {
    bin()
        .args([
            "bind",
            &fixture("valid/with_execution_model.dpcs.yaml"),
            "--profile",
            &fixture("capabilities/invalid/missing_mandatory.profile.yaml"),
            "--target",
            "dagster",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("DPCS-BIND-001"));
}

#[test]
fn bind_unknown_target_exits_one() {
    bin()
        .args([
            "bind",
            &fixture("valid/with_execution_model.dpcs.yaml"),
            "--profile",
            &fixture("capabilities/valid/matching.profile.yaml"),
            "--target",
            "make",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("DPCS-BIND-002"));
}

#[test]
fn bind_experimental_kubernetes() {
    let out = tempfile::tempdir().unwrap();
    bin()
        .args([
            "bind",
            &fixture("valid/with_execution_model.dpcs.yaml"),
            "--profile",
            &fixture("capabilities/valid/matching.profile.yaml"),
            "--target",
            "kubernetes",
            "--out",
            out.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("experimental: true"));
    assert!(out.path().join("cronjob.yaml").is_file());
}

#[test]
fn compatibility_cli_reports_category() {
    let baseline = format!(
        "{}/examples/compatibility/baseline.dpcs.yaml",
        env!("CARGO_MANIFEST_DIR")
    );
    let candidate = format!(
        "{}/examples/compatibility/candidate_compatible.dpcs.yaml",
        env!("CARGO_MANIFEST_DIR")
    );
    bin()
        .args(["compatibility", &baseline, &candidate])
        .assert()
        .success()
        .stdout(predicate::str::contains("category:"));
}

#[test]
fn registry_validate_cli() {
    let path = format!("{}/examples/registry.yaml", env!("CARGO_MANIFEST_DIR"));
    bin()
        .args(["registry", "validate", &path])
        .assert()
        .success();
}

#[test]
fn conformance_validate_cli() {
    let path = format!(
        "{}/examples/conformance.profile.yaml",
        env!("CARGO_MANIFEST_DIR")
    );
    bin()
        .args(["conformance", "validate", &path])
        .assert()
        .success();
}

#[test]
fn version_json_includes_conformance() {
    bin()
        .args(["version", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dpcsSpecVersion"))
        .stdout(predicate::str::contains("conformance"));
}
