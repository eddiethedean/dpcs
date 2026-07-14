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
fn rejects_duplicate_graph_edges() {
    let contract = parse_yaml_file(fixture("invalid/duplicate_edge.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-GRP-005"));
}

#[test]
fn rejects_unreachable_steps() {
    let contract = parse_yaml_file(fixture("invalid/unreachable_step.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-GRP-006"));
}

#[test]
fn rejects_invalid_entry_points() {
    let contract = parse_yaml_file(fixture("invalid/invalid_entry_point.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-GRP-007"));
    assert!(!report.diagnostics.iter().any(|d| d.id == "DPCS-GRP-006"));
}

#[test]
fn rejects_invalid_exit_points() {
    let contract = parse_yaml_file(fixture("invalid/invalid_exit_point.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-GRP-008"));
}

#[test]
fn rejects_bare_filename_contract_refs() {
    let contract = parse_yaml_file(fixture("invalid/bogus_filename_ref.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-REF-003"));
}

#[test]
fn rejects_unresolved_data_flow_contract_refs() {
    let contract = parse_yaml_file(fixture("invalid/unresolved_data_flow_ref.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report
        .diagnostics
        .iter()
        .any(|d| d.id == "DPCS-REF-003"
            && d.object_ref.as_deref() == Some("dataFlow[0].contractRef")));
}

#[test]
fn invalid_step_ports_do_not_emit_spurious_graph_errors() {
    let contract = parse_yaml_file(fixture("invalid/bad_step_port_endpoint.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-DF-002"));
    assert!(!report.diagnostics.iter().any(|d| d.id == "DPCS-GRP-004"));
    assert!(!report.diagnostics.iter().any(|d| d.id == "DPCS-GRP-006"));
}

#[test]
fn cycles_do_not_emit_unreachable_step_noise() {
    let contract = parse_yaml_file(fixture("invalid/cycle.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-GRP-004"));
    assert!(!report.diagnostics.iter().any(|d| d.id == "DPCS-GRP-006"));
}

#[test]
fn rejects_unresolved_contract_references() {
    let contract = parse_yaml_file(fixture("invalid/unresolved_ref.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-REF-003"));
}

#[test]
fn rejects_unresolved_transform_refs() {
    let contract = parse_yaml_file(fixture("invalid/unresolved_transform_ref.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-REF-005"));
}

#[test]
fn rejects_unresolved_step_port_refs() {
    let contract = parse_yaml_file(fixture("invalid/unresolved_step_port_ref.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-REF-006"));
}

#[test]
fn rejects_missing_dataset_identity() {
    let contract = parse_yaml_file(fixture("invalid/missing_dataset.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-DF-004"));
}

#[test]
fn rejects_unreachable_datasets() {
    let contract = parse_yaml_file(fixture("invalid/unreachable_dataset.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-DF-005"));
}

#[test]
fn rejects_unsatisfied_step_inputs() {
    let contract = parse_yaml_file(fixture("invalid/unsatisfied_step_input.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-DF-006"));
}

#[test]
fn rejects_illegal_data_flow_endpoint_roles() {
    let contract = parse_yaml_file(fixture("invalid/bad_flow_roles.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-DF-007"));
}

#[test]
fn cycle_does_not_suppress_unreachable_datasets() {
    let contract = parse_yaml_file(fixture("invalid/cycle_with_orphan_dataset.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-GRP-004"));
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-DF-005"));
}

#[test]
fn rejects_conflicting_control_and_graph_deps() {
    let contract = parse_yaml_file(fixture("invalid/conflicting_deps.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-CF-004"));
}

#[test]
fn rejects_duplicate_control_flow_edges() {
    let contract = parse_yaml_file(fixture("invalid/duplicate_control_flow.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-CF-005"));
}

#[test]
fn accepts_duplicate_graph_endpoints_with_different_kinds() {
    let contract =
        parse_yaml_file(fixture("valid/duplicate_edge_different_kind.dpcs.yaml")).unwrap();
    assert!(validate(&contract).is_valid());
}

#[test]
fn rejects_empty_step_port_ids() {
    let contract = parse_yaml_file(fixture("invalid/empty_step_port.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-STR-001"));
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

#[test]
fn validates_execution_model_contract() {
    let contract = parse_yaml_file(fixture("valid/with_execution_model.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.is_valid(), "{:?}", report.diagnostics);
}

#[test]
fn rejects_empty_required_capability() {
    let contract = parse_yaml_file(fixture("invalid/empty_capability.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-EXE-001"));
}

#[test]
fn rejects_incomplete_external_dependency() {
    let contract = parse_yaml_file(fixture("invalid/incomplete_external_dep.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-EXE-004"));
}

#[test]
fn rejects_scheduled_mode_without_frequency_or_cron() {
    let contract = parse_yaml_file(fixture("invalid/scheduled_missing_cron.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-SCH-002"));
}

#[test]
fn rejects_event_without_source() {
    let contract = parse_yaml_file(fixture("invalid/event_missing_source.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-SCH-005"));
}

#[test]
fn rejects_inconsistent_timing_constraints() {
    let contract = parse_yaml_file(fixture("invalid/timing_constraints.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-SCH-006"));
}

#[test]
fn rejects_incomplete_quality_gates() {
    let contract = parse_yaml_file(fixture("invalid/bad_quality_gate.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-QG-001"));
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-QG-002"));
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-QG-007"));
}

#[test]
fn rejects_unresolved_quality_gate_contract_ref() {
    let contract = parse_yaml_file(fixture("invalid/unresolved_qg_ref.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-QG-004"));
}

#[test]
fn rejects_invalid_failure_semantics() {
    let contract = parse_yaml_file(fixture("invalid/bad_failure_semantics.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-FS-003"));
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-FS-005"));
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-FS-007"));
}

#[test]
fn rejects_invalid_lineage_references() {
    let contract = parse_yaml_file(fixture("invalid/bad_lineage.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-LIN-004"));
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-LIN-010"));
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-LIN-012"));
}

#[test]
fn rejects_empty_retry_object() {
    let contract = parse_yaml_file(fixture("invalid/empty_retry.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-FS-007"));
}

#[test]
fn warns_on_freeform_timing_constraints() {
    let contract = parse_yaml_file(fixture("invalid/freeform_timing.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-SCH-007"));
    assert!(!report.diagnostics.iter().any(|d| d.id == "DPCS-SCH-006"));
}

#[test]
fn rejects_legacy_lineage_stub_fields() {
    let contract = parse_yaml_file(fixture("invalid/legacy_lineage.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-LIN-016"));
}

#[test]
fn rejects_invalid_lineage_warns_orphan_dataset() {
    let contract = parse_yaml_file(fixture("invalid/bad_lineage.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-LIN-002"));
}
