//! Appendix E conformance suite for ROADMAP 0.13.0.
//!
//! Maps recommended SPEC Appendix E areas to executable checks. Area coverage
//! is also documented in `docs/SPEC_COVERAGE.md`.

use dpcs::{
    bind, compare_contracts, compare_plans, evaluate, plan, plan_with_resolve,
    resolve_contract_references, toolkit_claim, validate, validate_claim,
    validate_conformance_profile, validate_diagnostic, validate_governance, validate_registry,
    validate_resolved, validate_security, BindingResult, BindingTarget, CapabilityProfile,
    ConformanceLevel, ConformanceProfile, Diagnostic, DiagnosticStage, GovernanceMetadata,
    PipelineContract, PlanResult, ResolveOptions, SecretReference, SecurityMetadata, Severity,
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

fn assert_has_id(report: &dpcs::ValidationReport, id: &str) {
    assert!(
        report.diagnostics.iter().any(|d| d.id == id),
        "expected {id}, got {:?}",
        report
            .diagnostics
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn appendix_e_parser_yaml_and_json() {
    let yaml = PipelineContract::from_yaml_file(fixture("valid/minimal.dpcs.yaml")).unwrap();
    let json = PipelineContract::from_json_file(fixture("valid/minimal.dpcs.json")).unwrap();
    assert_eq!(yaml.id, json.id);
    assert!(validate(&yaml).is_valid());
}

#[test]
fn appendix_e_com_and_validation() {
    let contract = PipelineContract::from_yaml_file(fixture("valid/minimal.dpcs.yaml")).unwrap();
    assert!(contract.identity().is_complete());
    let report = validate(&contract);
    assert!(report.is_valid());

    let bad =
        PipelineContract::from_yaml_file(fixture("invalid/duplicate_steps.dpcs.yaml")).unwrap();
    assert_has_id(&validate(&bad), "DPCS-COM-005");
}

#[test]
fn appendix_e_data_flow_and_control_flow() {
    assert_has_id(
        &validate(
            &PipelineContract::from_yaml_file(fixture("invalid/missing_dataset.dpcs.yaml"))
                .unwrap(),
        ),
        "DPCS-DF-004",
    );
    assert_has_id(
        &validate(
            &PipelineContract::from_yaml_file(fixture("invalid/conflicting_deps.dpcs.yaml"))
                .unwrap(),
        ),
        "DPCS-CF-004",
    );
}

#[test]
fn appendix_e_planning() {
    let contract = PipelineContract::from_yaml_file(example("with_execution.dpcs.yaml")).unwrap();
    let PlanResult::Ok(planned) = plan(&contract) else {
        panic!("expected plan");
    };
    assert!(!planned.step_order.is_empty());

    let bad =
        PipelineContract::from_yaml_file(fixture("invalid/duplicate_steps.dpcs.yaml")).unwrap();
    let PlanResult::Err(report) = plan(&bad) else {
        panic!("expected refusal");
    };
    assert_has_id(&report, "DPCS-PLN-001");
}

#[test]
fn appendix_e_capability_evaluation() {
    let contract =
        PipelineContract::from_yaml_file(fixture("valid/with_execution_model.dpcs.yaml")).unwrap();
    let planned = plan(&contract).plan().expect("plan");
    let matching =
        CapabilityProfile::from_yaml_file(fixture("capabilities/valid/matching.profile.yaml"))
            .unwrap();
    assert!(evaluate(&planned, &matching).is_ok());

    let missing = CapabilityProfile::from_yaml_file(fixture(
        "capabilities/invalid/missing_mandatory.profile.yaml",
    ))
    .unwrap();
    let result = evaluate(&planned, &missing);
    assert!(!result.is_ok());
}

#[test]
fn appendix_e_orchestrator_binding() {
    let contract =
        PipelineContract::from_yaml_file(fixture("valid/with_execution_model.dpcs.yaml")).unwrap();
    let planned = plan(&contract).plan().expect("plan");
    let profile =
        CapabilityProfile::from_yaml_file(fixture("capabilities/valid/matching.profile.yaml"))
            .unwrap();
    for target in BindingTarget::all() {
        let BindingResult::Ok(bundle) = bind(&planned, &profile, *target) else {
            panic!("bind failed for {target}");
        };
        assert!(
            bundle
                .files
                .iter()
                .any(|f| f.relative_path == "dpcs_semantics.json"),
            "{target} missing dpcs_semantics.json"
        );
        let semantics = bundle
            .files
            .iter()
            .find(|f| f.relative_path == "dpcs_semantics.json")
            .unwrap();
        assert!(semantics.content.contains("\"scheduling\""));
        assert!(semantics.content.contains("\"qualityGates\""));
        assert!(semantics.content.contains("\"failureSemantics\""));
    }
}

#[test]
fn appendix_e_diagnostics_meta() {
    let empty = Diagnostic::error("", "document", "x");
    let report = validate_diagnostic(&empty);
    assert_has_id(&report, "DPCS-DIAG-001");

    let bad_prefix = Diagnostic::error("TEST-001", "document", "x");
    assert_has_id(&validate_diagnostic(&bad_prefix), "DPCS-DIAG-002");

    let no_category = Diagnostic::error("DPCS-TEST-001", "", "x");
    assert_has_id(&validate_diagnostic(&no_category), "DPCS-DIAG-003");

    let no_message = Diagnostic::error("DPCS-TEST-001", "document", "");
    assert_has_id(&validate_diagnostic(&no_message), "DPCS-DIAG-004");
}

#[test]
fn appendix_e_compatibility() {
    let baseline =
        PipelineContract::from_yaml_file(example("compatibility/baseline.dpcs.yaml")).unwrap();
    let compatible =
        PipelineContract::from_yaml_file(example("compatibility/candidate_compatible.dpcs.yaml"))
            .unwrap();
    let breaking =
        PipelineContract::from_yaml_file(example("compatibility/candidate_breaking.dpcs.yaml"))
            .unwrap();
    assert!(compare_contracts(&baseline, &compatible).is_ok());
    assert!(!compare_contracts(&baseline, &breaking).is_ok());

    let p1 = plan(&baseline).plan().expect("plan baseline");
    let p2 = plan(&compatible).plan().expect("plan compatible");
    let plan_compat = compare_plans(&p1, &p2);
    assert!(
        plan_compat.is_ok(),
        "additive plans should be compatible: {:?}",
        plan_compat.report()
    );
    assert!(plan_compat.report().category.is_compatible());
    // Breaking candidate is invalid as a plan target (removed interface output);
    // contract comparison covers the incompatibility SHALL.
    assert!(!compare_contracts(&baseline, &breaking).is_ok());
}

#[test]
fn appendix_e_versioning_and_extensions() {
    assert_has_id(
        &validate(
            &PipelineContract::from_yaml_file(fixture("invalid/invalid_version.dpcs.yaml"))
                .unwrap(),
        ),
        "DPCS-VER-001",
    );
    assert_has_id(
        &validate(
            &PipelineContract::from_yaml_file(fixture("invalid/bad_extension_namespace.dpcs.yaml"))
                .unwrap(),
        ),
        "DPCS-EXT-001",
    );
}

#[test]
fn appendix_e_registries() {
    let registry = dpcs::Registry::from_file(example("registry.yaml")).unwrap();
    assert!(validate_registry(&registry).is_valid());
    let invalid =
        dpcs::Registry::from_file(fixture("invalid/duplicate_registry_artifact.yaml")).unwrap();
    assert_has_id(&validate_registry(&invalid), "DPCS-REG-014");
}

#[test]
fn appendix_e_nested_resolve_and_plan() {
    let parent =
        PipelineContract::from_yaml_file(fixture("valid/nested/parent.dpcs.yaml")).unwrap();
    let opts = ResolveOptions::from_document_path(fixture("valid/nested/parent.dpcs.yaml"));
    let resolution = resolve_contract_references(&parent, &opts);
    assert!(resolution.is_ok(), "{:?}", resolution.report.diagnostics);
    assert_eq!(resolution.nested.len(), 1);
    assert_eq!(resolution.nested[0].contract.id, "nested.child");

    let PlanResult::Ok(planned) = plan_with_resolve(&parent, Some(&opts)) else {
        panic!("expected nested plan");
    };
    assert_eq!(planned.nested.len(), 1);
    assert_eq!(planned.nested[0].contract_id, "nested.child");
    let lineage = planned.lineage.as_ref().expect("lineage");
    let provenance = lineage.provenance.as_ref().expect("provenance");
    assert!(provenance.nested.contains(&"nested.child".to_string()));

    let missing =
        PipelineContract::from_yaml_file(fixture("invalid/nested_missing.dpcs.yaml")).unwrap();
    let opts = ResolveOptions::from_document_path(fixture("invalid/nested_missing.dpcs.yaml"));
    let report = validate_resolved(&missing, &opts);
    assert_has_id(&report, "DPCS-REF-007");

    // Default plan() resolves relative to CWD (document-relative needs chdir or opts).
    let nested_dir = fixture("valid/nested");
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&nested_dir).unwrap();
    let parent_cwd =
        PipelineContract::from_yaml_file("parent.dpcs.yaml").expect("load parent from cwd");
    let PlanResult::Ok(planned_default) = plan(&parent_cwd) else {
        let _ = std::env::set_current_dir(&prev);
        panic!("default plan() should resolve nested from CWD");
    };
    let _ = std::env::set_current_dir(&prev);
    assert_eq!(planned_default.nested[0].contract_id, "nested.child");

    // Recursive nest preserves children and interface port lists.
    let deep =
        PipelineContract::from_yaml_file(fixture("valid/nested/parent_deep.dpcs.yaml")).unwrap();
    let opts = ResolveOptions::from_document_path(fixture("valid/nested/parent_deep.dpcs.yaml"));
    let PlanResult::Ok(planned_deep) = plan_with_resolve(&deep, Some(&opts)) else {
        let PlanResult::Err(report) = plan_with_resolve(&deep, Some(&opts)) else {
            unreachable!();
        };
        panic!("expected deep nested plan: {:?}", report.diagnostics);
    };
    assert_eq!(planned_deep.nested[0].contract_id, "nested.mid");
    assert!(planned_deep.nested[0].input_ports.is_empty());
    assert!(planned_deep.nested[0].output_ports.is_empty());
    assert_eq!(planned_deep.nested[0].children.len(), 1);
    assert_eq!(
        planned_deep.nested[0].children[0].contract_id,
        "nested.grandchild"
    );
    let lineage = planned_deep.lineage.as_ref().unwrap();
    let provenance = lineage.provenance.as_ref().unwrap();
    assert!(provenance.nested.contains(&"nested.mid".to_string()));
    assert!(provenance.nested.contains(&"nested.grandchild".to_string()));

    let cycle =
        PipelineContract::from_yaml_file(fixture("invalid/nested_cycle_a.dpcs.yaml")).unwrap();
    let opts = ResolveOptions::from_document_path(fixture("invalid/nested_cycle_a.dpcs.yaml"));
    let report = validate_resolved(&cycle, &opts);
    assert_has_id(&report, "DPCS-REF-008");
}

#[test]
fn appendix_e_security_governance_profiles() {
    let mut security = SecurityMetadata::default();
    security.secret_refs.push(SecretReference {
        id: "x".into(),
        loc: "embeddedsupersecrettoken".into(),
        provider: None,
        extensions: Default::default(),
    });
    assert_has_id(&validate_security(&security), "DPCS-SEC-003");

    let gov = GovernanceMetadata::default();
    let _ = validate_governance(&gov);

    let profile = ConformanceProfile {
        id: String::new(),
        version: "0.1.0".into(),
        dpcs_version: "1.0.0-draft".into(),
        levels: vec![ConformanceLevel::Parser],
        forbidden_extension_namespaces: vec![],
        require_security: false,
        require_governance: false,
        extensions: Default::default(),
    };
    assert_has_id(&validate_conformance_profile(&profile), "DPCS-CONF-001");

    let claim = toolkit_claim();
    assert!(validate_claim(&claim).is_valid());
    assert!(claim
        .levels
        .contains(&ConformanceLevel::CompleteImplementation));
}

#[test]
fn appendix_e_diagnostic_stages_present_for_refusals() {
    let bad =
        PipelineContract::from_yaml_file(fixture("invalid/duplicate_steps.dpcs.yaml")).unwrap();
    let PlanResult::Err(report) = plan(&bad) else {
        panic!("expected planning refusal");
    };
    assert!(report
        .diagnostics
        .iter()
        .any(|d| d.id == "DPCS-PLN-001" && d.stage == DiagnosticStage::Planning));
    assert!(report
        .diagnostics
        .iter()
        .any(|d| d.severity == Severity::Error));
}
