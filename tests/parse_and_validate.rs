//! Integration tests for parsing and validation.

use dpcs::{parse_json, parse_yaml, parse_yaml_file, validate};

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(path)
}

#[test]
fn parses_valid_minimal_yaml() {
    let contract = parse_yaml_file(fixture("valid/minimal.dpcs.yaml")).unwrap();
    assert_eq!(contract.id, "valid.minimal");
    assert!(contract.validate().is_valid());
}

#[test]
fn parses_valid_minimal_json() {
    let input = include_str!("fixtures/valid/minimal.dpcs.json");
    let contract = parse_json(input).unwrap();
    assert_eq!(contract.id, "valid.minimal");
    assert!(validate(&contract).is_valid());
}

#[test]
fn rejects_malformed_yaml() {
    let err = parse_yaml(":\n").unwrap_err();
    let report = err.invalid_document_report().expect("parse report");
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-PARSE-001"));
}

#[test]
fn rejects_missing_required_fields() {
    let err = parse_yaml("id: only-id\n").unwrap_err();
    let report = err.invalid_document_report().expect("parse report");
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-PARSE-002"));
}

#[test]
fn rejects_duplicate_step_identifiers() {
    let contract = parse_yaml_file(fixture("invalid/duplicate_steps.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-COM-005"));
    assert_eq!(
        report
            .diagnostics
            .iter()
            .filter(|d| d.id == "DPCS-COM-005")
            .count(),
        1
    );
}

#[test]
fn rejects_invalid_graph_edges() {
    let contract = parse_yaml_file(fixture("invalid/unknown_edge.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-GRP-003"));
}

#[test]
fn rejects_prohibited_cycles() {
    let contract = parse_yaml_file(fixture("invalid/cycle.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-GRP-004"));
}

#[test]
fn rejects_unresolved_contract_references() {
    let contract = parse_yaml_file(fixture("invalid/unresolved_ref.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-REF-003"));
}

#[test]
fn validates_data_flow_endpoints() {
    let contract = parse_yaml_file(fixture("valid/with_data_flow.dpcs.yaml")).unwrap();
    assert!(validate(&contract).is_valid());

    let invalid = parse_yaml_file(fixture("invalid/bad_data_flow.dpcs.yaml")).unwrap();
    let report = validate(&invalid);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-DF-002"));
}

#[test]
fn validates_control_flow_dependencies() {
    let invalid = parse_yaml_file(fixture("invalid/bad_control_flow.dpcs.yaml")).unwrap();
    let report = validate(&invalid);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-CF-002"));
}

#[test]
fn preserves_extension_fields() {
    let input = r#"
dpcsVersion: "1.0.0"
id: "ext.pipeline"
version: "0.1.0"
interface:
  inputs: []
  outputs: []
steps: []
graph:
  edges: []
x-vendor:
  team: data-platform
"#;
    let contract = parse_yaml(input).unwrap();
    assert_eq!(
        contract.extensions["x-vendor"]
            .get("team")
            .and_then(|v| v.as_str()),
        Some("data-platform")
    );
    assert!(validate(&contract).is_valid());
}

#[test]
fn rejects_duplicate_interface_ports_across_sides() {
    let contract = parse_yaml_file(fixture("invalid/duplicate_interface_ports.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-COM-013"));
    assert!(!report.diagnostics.iter().any(|d| d.id == "DPCS-COM-005"));
}

#[test]
fn rejects_duplicate_interface_inputs_with_com_005_only() {
    let contract =
        parse_yaml_file(fixture("invalid/duplicate_interface_inputs.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-COM-005"));
    assert!(!report.diagnostics.iter().any(|d| d.id == "DPCS-COM-013"));
}

#[test]
fn rejects_undeclared_step_port_endpoints() {
    let contract = parse_yaml_file(fixture("invalid/bad_step_port_endpoint.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-DF-002"));
}

#[test]
fn diagnostics_are_deterministic() {
    let contract = parse_yaml_file(fixture("invalid/duplicate_steps.dpcs.yaml")).unwrap();
    let first = validate(&contract);
    let second = validate(&contract);
    assert_eq!(first, second);
}
