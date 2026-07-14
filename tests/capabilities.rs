//! Capability profile evaluation tests (ROADMAP 0.7.0).

use dpcs::{
    evaluate, evaluate_many, evaluate_requirements, parse_yaml_file, plan, try_plan,
    validate_profile, CapabilityProfile, CapabilityResult, PlanResult,
};

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(path)
}

#[test]
fn matching_profile_satisfies_execution_model_plan() {
    let contract = parse_yaml_file(fixture("valid/with_execution_model.dpcs.yaml")).unwrap();
    let PlanResult::Ok(planned) = plan(&contract) else {
        panic!("expected plan");
    };
    let profile =
        CapabilityProfile::from_yaml_file(fixture("capabilities/valid/matching.profile.yaml"))
            .unwrap();

    let CapabilityResult::Ok(report) = evaluate(&planned, &profile) else {
        panic!("expected capability match");
    };
    assert_eq!(report.profile_identity, "reference.batch");
    assert!(report.satisfied.iter().any(|id| id == "batch.compute"));
    assert!(report.satisfied.iter().any(|id| id == "sql.readwrite"));
    assert!(report.missing_mandatory.is_empty());
    assert!(report
        .unsupported_optional
        .iter()
        .any(|id| id == "observability.metrics"));
}

#[test]
fn missing_mandatory_capability_is_rejected() {
    let contract = parse_yaml_file(fixture("valid/with_execution_model.dpcs.yaml")).unwrap();
    let planned = try_plan(&contract).expect("plan");
    let profile = CapabilityProfile::from_yaml_file(fixture(
        "capabilities/invalid/missing_mandatory.profile.yaml",
    ))
    .unwrap();

    let CapabilityResult::Err(report) = evaluate(&planned, &profile) else {
        panic!("expected capability failure");
    };
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-CAP-005"));
    assert!(report
        .diagnostics
        .iter()
        .any(|d| d.stage.to_string() == "capabilityEvaluation"));
}

#[test]
fn validate_profile_rejects_empty_and_duplicate_ids() {
    let profile =
        CapabilityProfile::from_yaml_file(fixture("capabilities/invalid/bad_profile.profile.yaml"))
            .unwrap();
    let report = validate_profile(&profile);
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-CAP-001"));
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-CAP-002"));
}

#[test]
fn evaluate_requirements_matches_without_plan() {
    let contract = parse_yaml_file(fixture("valid/with_execution_model.dpcs.yaml")).unwrap();
    let requirements = contract.execution.as_ref().expect("execution");
    let profile =
        CapabilityProfile::from_yaml_file(fixture("capabilities/valid/matching.profile.yaml"))
            .unwrap();
    assert!(evaluate_requirements(requirements, &profile).is_ok());
}

#[test]
fn evaluate_many_ranks_profiles() {
    let contract = parse_yaml_file(fixture("valid/with_execution_model.dpcs.yaml")).unwrap();
    let planned = try_plan(&contract).expect("plan");
    let matching =
        CapabilityProfile::from_yaml_file(fixture("capabilities/valid/matching.profile.yaml"))
            .unwrap();
    let incomplete = CapabilityProfile::from_yaml_file(fixture(
        "capabilities/invalid/missing_mandatory.profile.yaml",
    ))
    .unwrap();

    let results = evaluate_many(&planned, &[matching, incomplete]);
    assert_eq!(results.len(), 2);
    assert!(results[0].1.is_ok());
    assert!(!results[1].1.is_ok());
}

#[test]
fn version_mismatch_warns_but_can_still_match() {
    let contract = parse_yaml_file(fixture("valid/with_execution_model.dpcs.yaml")).unwrap();
    let planned = try_plan(&contract).expect("plan");
    let profile = CapabilityProfile::from_yaml_file(fixture(
        "capabilities/valid/version_mismatch.profile.yaml",
    ))
    .unwrap();

    let CapabilityResult::Ok(report) = evaluate(&planned, &profile) else {
        panic!("expected match with warning");
    };
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-CAP-006"));
}

#[test]
fn profile_round_trip_preserves_com() {
    let profile =
        CapabilityProfile::from_yaml_file(fixture("capabilities/valid/matching.profile.yaml"))
            .unwrap();
    let yaml = profile.to_yaml_str().unwrap();
    let reparsed = CapabilityProfile::from_yaml_str(&yaml).unwrap();
    assert_eq!(profile, reparsed);
    let json = profile.to_json_str().unwrap();
    assert_eq!(profile, CapabilityProfile::from_json_str(&json).unwrap());
}

#[test]
fn legacy_profile_alias_deserializes_as_identity() {
    let profile = CapabilityProfile::from_yaml_str(
        r#"
profile: "legacy.name"
dpcsVersion: "1.0.0-draft"
capabilities:
  - id: "batch.compute"
"#,
    )
    .unwrap();
    assert_eq!(profile.identity, "legacy.name");
}
