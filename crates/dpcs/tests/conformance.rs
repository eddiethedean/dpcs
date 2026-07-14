//! Appendix E-aligned conformance suite for ROADMAP 0.9.0.

use dpcs::{
    compare_contracts, plan, toolkit_claim, validate, validate_claim, validate_conformance_profile,
    validate_diagnostic, validate_registry, ConformanceProfile, Diagnostic, DiagnosticReport,
    PipelineContract, PlanResult, Registry, RegistryReference,
};

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(path)
}

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn example(path: &str) -> std::path::PathBuf {
    workspace_root().join("examples").join(path)
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
    assert_eq!(
        result.report().category,
        dpcs::CompatibilityCategory::BackwardCompatible
    );
}

#[test]
fn planner_capability_and_binding_levels_still_work() {
    let contract = PipelineContract::from_yaml_file(example("with_execution.dpcs.yaml")).unwrap();
    assert!(plan(&contract).is_ok());
}

#[test]
fn apply_profile_requires_governance() {
    use dpcs::{apply_profile_to_contract, ConformanceLevel, ConformanceProfile};

    let contract = PipelineContract::from_yaml_file(fixture("valid/minimal.dpcs.yaml")).unwrap();
    let profile = ConformanceProfile {
        id: "req.gov".into(),
        version: "0.1.0".into(),
        dpcs_version: "1.0.0-draft".into(),
        levels: vec![ConformanceLevel::Validator],
        forbidden_extension_namespaces: vec![],
        require_security: false,
        require_governance: true,
        extensions: Default::default(),
    };
    let report = apply_profile_to_contract(&contract, &profile);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-CONF-013"));
}

#[test]
fn planning_refusal_links_related_validation_errors() {
    let contract =
        PipelineContract::from_yaml_file(fixture("invalid/duplicate_steps.dpcs.yaml")).unwrap();
    let PlanResult::Err(report) = plan(&contract) else {
        panic!("expected planning refusal");
    };
    let pln = report
        .diagnostics
        .iter()
        .find(|d| d.id == "DPCS-PLN-001")
        .expect("PLN-001");
    assert!(
        pln.related_diagnostics
            .iter()
            .any(|id| id == "DPCS-COM-005"),
        "expected related COM-005, got {:?}",
        pln.related_diagnostics
    );
}

#[test]
fn security_rejects_embedded_secret_material() {
    use dpcs::validate_security;
    use dpcs::{SecretReference, SecurityMetadata};

    let security = SecurityMetadata {
        secret_refs: vec![SecretReference {
            id: "db".into(),
            loc: "supersecrettokenvalue".into(),
            provider: None,
            extensions: Default::default(),
        }],
        ..Default::default()
    };
    let report = validate_security(&security);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-SEC-003"));
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
