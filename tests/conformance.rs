//! Appendix E-aligned conformance suite for ROADMAP 0.9.0.

use dpcs::{
    compare_contracts, plan, toolkit_claim, validate, validate_claim, validate_conformance_profile,
    validate_diagnostic, validate_registry, ConformanceProfile, Diagnostic, DiagnosticReport,
    PipelineContract, Registry, RegistryReference,
};

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(path)
}

fn example(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(path)
}

#[test]
fn parser_and_validator_accept_minimal_contract() {
    let contract = PipelineContract::from_yaml_file(fixture("valid/minimal.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.is_valid());
}

#[test]
fn diagnostics_support_related_and_report_wrapper() {
    let diagnostic =
        Diagnostic::error("DPCS-TEST-001", "document", "sample").with_related(["DPCS-TEST-002"]);
    assert_eq!(diagnostic.related_diagnostics, ["DPCS-TEST-002"]);
    let shape = validate_diagnostic(&diagnostic);
    assert!(shape.is_valid());

    let mut report = dpcs::ValidationReport::new();
    report.push(diagnostic);
    let wrapped = DiagnosticReport::from_validation(report, Some("artifact".into()));
    assert!(!wrapped.is_valid());
    assert_eq!(wrapped.implementation.name, "dpcs");
}

#[test]
fn versioning_rejects_invalid_semver() {
    let contract =
        PipelineContract::from_yaml_file(fixture("invalid/invalid_version.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-VER-001"));
}

#[test]
fn extensions_reject_invalid_namespace_and_preserve_valid() {
    let bad =
        PipelineContract::from_yaml_file(fixture("invalid/bad_extension_namespace.dpcs.yaml"))
            .unwrap();
    let report = validate(&bad);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-EXT-001"));

    let good = PipelineContract::from_yaml_file(fixture("valid/with_nested_extensions.dpcs.yaml"))
        .unwrap();
    let report = validate(&good);
    assert!(report.is_valid());
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-EXT-010"));
}

#[test]
fn compatibility_detects_breaking_interface_removal() {
    let baseline =
        PipelineContract::from_yaml_file(example("compatibility/baseline.dpcs.yaml")).unwrap();
    let breaking =
        PipelineContract::from_yaml_file(example("compatibility/candidate_breaking.dpcs.yaml"))
            .unwrap();
    let result = compare_contracts(&baseline, &breaking);
    assert!(!result.is_ok());
    assert!(result
        .report()
        .diagnostics
        .iter()
        .any(|d| d.id == "DPCS-COMPAT-002"));
}

#[test]
fn compatibility_allows_additive_steps() {
    let baseline =
        PipelineContract::from_yaml_file(example("compatibility/baseline.dpcs.yaml")).unwrap();
    let candidate =
        PipelineContract::from_yaml_file(example("compatibility/candidate_compatible.dpcs.yaml"))
            .unwrap();
    let result = compare_contracts(&baseline, &candidate);
    assert!(result.is_ok());
    assert!(result.report().category.is_compatible());
}

#[test]
fn planner_capability_and_binding_levels_still_work() {
    let contract = PipelineContract::from_yaml_file(example("with_execution.dpcs.yaml")).unwrap();
    assert!(plan(&contract).is_ok());
}

#[test]
fn registry_validates_documents_and_duplicates() {
    let registry = Registry::from_file(example("registry.yaml")).unwrap();
    let report = validate_registry(&registry);
    assert!(report.is_valid());

    let invalid = Registry::from_file(fixture("invalid/duplicate_registry_artifact.yaml")).unwrap();
    let report = validate_registry(&invalid);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-REG-014"));

    let _ = RegistryReference::default();
}

#[test]
fn security_and_governance_validate() {
    let contract =
        PipelineContract::from_yaml_file(example("with_security_governance.dpcs.yaml")).unwrap();
    let report = validate(&contract);
    assert!(report.is_valid(), "{:?}", report.diagnostics);
    assert!(contract.security.is_some());
    assert!(contract.governance.is_some());
}

#[test]
fn conformance_profile_and_toolkit_claim() {
    let profile = ConformanceProfile::from_file(example("conformance.profile.yaml")).unwrap();
    let report = validate_conformance_profile(&profile);
    assert!(report.is_valid());

    let claim = toolkit_claim();
    let report = validate_claim(&claim);
    assert!(report.is_valid());
    assert!(!claim.levels.is_empty());
}
