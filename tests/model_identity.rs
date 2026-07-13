//! Model identity and COM invariant tests.

use dpcs::{ExtensionValue, IdentityCatalog, ObjectKind, PipelineContract};

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(path)
}

#[test]
fn pipeline_identity_extracts_root_fields() {
    let contract =
        PipelineContract::from_yaml_file(fixture("valid/with_data_flow.dpcs.yaml")).unwrap();
    let identity = contract.identity();

    assert_eq!(identity.id.as_str(), "valid.data.flow");
    assert_eq!(identity.version.as_str(), "0.1.0");
    assert_eq!(identity.dpcs_version.as_str(), "1.0.0");
    assert!(identity.is_complete());
}

#[test]
fn identity_catalog_lists_addressable_objects() {
    let contract =
        PipelineContract::from_yaml_file(fixture("valid/with_data_flow.dpcs.yaml")).unwrap();
    let catalog = contract.identity_catalog();

    assert!(catalog.get_by_path("pipeline").is_some());
    assert!(catalog
        .get_by_kind_and_id(ObjectKind::Step, "normalize_customer")
        .is_some());
    assert!(catalog
        .get_by_path("interface.inputs.customer_raw")
        .is_some());
    assert!(contract.interface.has_unique_port_ids());
}

#[test]
fn yaml_and_json_produce_equal_com_values() {
    let yaml = PipelineContract::from_yaml_file(fixture("valid/minimal.dpcs.yaml")).unwrap();
    let json = PipelineContract::from_json_file(fixture("valid/minimal.dpcs.json")).unwrap();
    assert_eq!(yaml, json);
}

#[test]
fn com_invariants_report_incomplete_interface_ports() {
    let contract =
        PipelineContract::from_yaml_file(fixture("invalid/incomplete_interface_port.dpcs.yaml"))
            .unwrap();
    let report = contract.validate();

    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-COM-006"));
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-COM-008"));
    assert!(report
        .diagnostics
        .iter()
        .any(|d| d.stage.to_string() == "canonicalObjectModel"));
}

#[test]
fn extension_value_round_trips_through_serde_json() {
    let json = serde_json::json!({
        "team": "data-platform",
        "count": 3
    });

    let extension: ExtensionValue = json.clone().into();
    let back: serde_json::Value = extension.into();
    assert_eq!(json, back);
}

#[test]
fn duplicate_step_ids_surface_in_com_diagnostics() {
    let contract =
        PipelineContract::from_yaml_file(fixture("invalid/duplicate_steps.dpcs.yaml")).unwrap();
    let catalog = IdentityCatalog::from_contract(&contract);
    let duplicates = catalog.duplicate_ids_by_kind();

    assert!(duplicates.contains_key(&ObjectKind::Step));
    assert!(contract
        .validate()
        .diagnostics
        .iter()
        .any(|d| d.id == "DPCS-COM-005"));
}
