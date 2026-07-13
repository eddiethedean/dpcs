//! Round-trip and parse diagnostic tests.

use dpcs::{
    parse_file, parse_json, parse_yaml, parse_yaml_file, to_json, to_yaml, DiagnosticStage, Error,
    PipelineContract,
};

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(path)
}

#[test]
fn yaml_round_trip_preserves_com() {
    let original = parse_yaml_file(fixture("valid/minimal.dpcs.yaml")).unwrap();
    let serialized = to_yaml(&original).unwrap();
    let round_tripped = parse_yaml(&serialized).unwrap();
    assert_eq!(original, round_tripped);
}

#[test]
fn json_round_trip_preserves_com() {
    let original = parse_yaml_file(fixture("valid/minimal.dpcs.yaml")).unwrap();
    let serialized = to_json(&original).unwrap();
    let round_tripped = parse_json(&serialized).unwrap();
    assert_eq!(original, round_tripped);
}

#[test]
fn cross_format_yaml_to_json_preserves_com() {
    let original = parse_yaml_file(fixture("valid/minimal.dpcs.yaml")).unwrap();
    let json = to_json(&original).unwrap();
    let from_json = parse_json(&json).unwrap();
    assert_eq!(original, from_json);
}

#[test]
fn nested_extensions_survive_round_trip() {
    let original = parse_yaml_file(fixture("valid/with_nested_extensions.dpcs.yaml")).unwrap();
    assert_eq!(
        original
            .interface
            .extensions
            .get("x-interface")
            .and_then(|v| v.get("owner"))
            .and_then(|v| v.as_str()),
        Some("data-platform")
    );
    assert_eq!(
        original.steps[0]
            .extensions
            .get("x-step")
            .and_then(|v| v.get("priority"))
            .and_then(|v| v.as_str()),
        Some("high")
    );
    assert_eq!(
        original
            .graph
            .extensions
            .get("x-graph")
            .and_then(|v| v.get("layout"))
            .and_then(|v| v.as_str()),
        Some("linear")
    );

    let round_tripped = parse_yaml(&to_yaml(&original).unwrap()).unwrap();
    assert_eq!(original, round_tripped);
}

#[test]
fn malformed_yaml_emits_parse_stage_diagnostic() {
    let err = parse_yaml(":\n").unwrap_err();
    let report = err.invalid_document_report().expect("parse report");
    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-PARSE-001"));
    assert!(report
        .diagnostics
        .iter()
        .all(|d| d.stage == DiagnosticStage::Parse));
}

#[test]
fn malformed_json_emits_parse_stage_diagnostic() {
    let err = parse_json("{ invalid json").unwrap_err();
    let report = err.invalid_document_report().expect("parse report");
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-PARSE-001"));
}

#[test]
fn missing_required_fields_emits_parse_diagnostic() {
    let err = parse_yaml("id: only-id\n").unwrap_err();
    let report = err.invalid_document_report().expect("parse report");
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-PARSE-002"));
}

#[test]
fn invalid_field_type_emits_schema_parse_diagnostic() {
    let err = parse_yaml(
        r#"
dpcsVersion: ["1.0.0"]
id: "typed"
version: "0.1.0"
interface:
  inputs: []
  outputs: []
graph:
  edges: []
"#,
    )
    .unwrap_err();
    let report = err.invalid_document_report().expect("parse report");
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|d| d.id == "DPCS-PARSE-002")
        .expect("typed field diagnostic");
    assert!(diagnostic
        .remediation
        .as_deref()
        .is_some_and(|text| text.contains("schema types")));
}

#[test]
fn file_parse_failure_includes_path_in_source_location() {
    let path = fixture("invalid/malformed.dpcs.yaml");
    let err = parse_yaml_file(&path).unwrap_err();
    let report = err.invalid_document_report().expect("parse report");
    let location = report.diagnostics[0]
        .source_location
        .as_deref()
        .expect("source location");
    assert!(location.contains("malformed.dpcs.yaml"));
}

#[test]
fn reserved_extension_keys_are_omitted_from_wire_serialization() {
    let mut contract = parse_yaml_file(fixture("valid/minimal.dpcs.yaml")).unwrap();
    contract.extensions.insert(
        "id".to_owned(),
        dpcs::ExtensionValue::String("collision".to_owned()),
    );

    let yaml = to_yaml(&contract).unwrap();
    let reparsed = parse_yaml(&yaml).unwrap();
    assert_eq!(reparsed.id, contract.id);
    assert!(!reparsed.extensions.contains_key("id"));
    assert!(!yaml.contains("collision"));

    let report = contract.validate();
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-COM-012"));
}

#[test]
fn unsupported_format_is_hard_error() {
    let err = parse_file(fixture("valid/minimal.dpcs.toml")).unwrap_err();
    assert!(matches!(err, Error::UnsupportedFormat { .. }));
    assert!(err.invalid_document_report().is_none());
}

#[test]
fn pipeline_contract_serialize_methods_match_parser() {
    let contract = PipelineContract::from_yaml_file(fixture("valid/minimal.dpcs.yaml")).unwrap();
    let yaml = contract.to_yaml_str().unwrap();
    let json = contract.to_json_str().unwrap();
    assert_eq!(contract, parse_yaml(&yaml).unwrap());
    assert_eq!(contract, parse_json(&json).unwrap());
}
