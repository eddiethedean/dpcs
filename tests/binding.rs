//! Orchestrator binding tests (ROADMAP 0.8.0).

use dpcs::{
    bind, bind_contract, parse_target, parse_yaml_file, plan, try_plan, write_bundle,
    BindingBundle, BindingFile, BindingFramework, BindingResult, BindingTarget, CapabilityProfile,
    DiagnosticStage, PlanResult,
};
use std::collections::BTreeSet;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(path)
}

fn matching_profile() -> CapabilityProfile {
    CapabilityProfile::from_yaml_file(fixture("capabilities/valid/matching.profile.yaml")).unwrap()
}

fn execution_plan() -> dpcs::PipelinePlan {
    let contract = parse_yaml_file(fixture("valid/with_execution_model.dpcs.yaml")).unwrap();
    try_plan(&contract).expect("plan")
}

#[test]
fn binding_framework_is_available() {
    assert!(BindingFramework::is_available());
    assert_eq!(BindingFramework::supported_targets().len(), 5);
}

#[test]
fn parse_target_rejects_unknown() {
    let err = parse_target("make").unwrap_err();
    assert!(err.diagnostics.iter().any(|d| d.id == "DPCS-BIND-002"));
    assert!(err
        .diagnostics
        .iter()
        .any(|d| d.stage == DiagnosticStage::OrchestratorBinding));
}

#[test]
fn parse_target_accepts_k8s_alias() {
    assert_eq!(parse_target("k8s").unwrap(), BindingTarget::Kubernetes);
}

#[test]
fn bind_all_targets_preserve_identity_and_steps() {
    let planned = execution_plan();
    let profile = matching_profile();

    for target in BindingTarget::all() {
        let BindingResult::Ok(bundle) = bind(&planned, &profile, *target) else {
            panic!("expected bind success for {target}");
        };
        assert_eq!(bundle.target, *target);
        assert_eq!(bundle.contract_id, "valid.execution.model");
        assert_eq!(bundle.profile_identity, "reference.batch");
        assert!(!bundle.files.is_empty());

        let content = bundle
            .files
            .iter()
            .map(|f| f.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            content.contains("valid.execution.model"),
            "{target} missing contract id"
        );
        assert!(
            content.contains("normalize_customer"),
            "{target} missing step id"
        );
        assert!(
            content.contains("reference.batch") || content.contains("reference-batch"),
            "{target} missing profile identity"
        );
    }
}

#[test]
fn bind_multi_step_encodes_dependencies() {
    let contract = parse_yaml_file(fixture("valid/with_graph_features.dpcs.yaml")).unwrap();
    let PlanResult::Ok(planned) = plan(&contract) else {
        panic!("expected plan");
    };
    assert!(planned.step_order.len() >= 3);
    assert!(!planned.dependency_edges.is_empty());

    let profile = matching_profile();
    let BindingResult::Ok(bundle) = bind(&planned, &profile, BindingTarget::Airflow) else {
        panic!("expected airflow bind");
    };
    let content = &bundle.files[0].content;
    assert!(content.contains("ingest >> transform") || content.contains("ingest >>"));
    assert!(content.contains("transform >> publish") || content.contains(">> publish"));

    let BindingResult::Ok(dagster) = bind(&planned, &profile, BindingTarget::Dagster) else {
        panic!("expected dagster bind");
    };
    let dagster_content = &dagster.files[0].content;
    assert!(
        dagster_content.contains("transform_op(ingest_result)")
            || dagster_content.contains("_op(ingest_result)")
    );
}

#[test]
fn airflow_does_not_invent_edges_when_independent() {
    let contract = parse_yaml_file(fixture("valid/minimal.dpcs.yaml")).unwrap();
    let planned = try_plan(&contract).expect("plan");
    assert!(planned.dependency_edges.is_empty());
    let profile = matching_profile();
    let BindingResult::Ok(bundle) = bind(&planned, &profile, BindingTarget::Airflow) else {
        panic!("expected bind");
    };
    assert!(
        !bundle.files[0].content.contains(" >> "),
        "independent plans must not invent Airflow dependencies"
    );
}

#[test]
fn bind_refuses_missing_mandatory_capability() {
    let planned = execution_plan();
    let profile = CapabilityProfile::from_yaml_file(fixture(
        "capabilities/invalid/missing_mandatory.profile.yaml",
    ))
    .unwrap();

    let BindingResult::Err {
        diagnostics,
        capability,
    } = bind(&planned, &profile, BindingTarget::Airflow)
    else {
        panic!("expected capability refusal");
    };
    assert!(diagnostics
        .diagnostics
        .iter()
        .any(|d| d.id == "DPCS-BIND-001"));
    assert!(diagnostics
        .diagnostics
        .iter()
        .any(|d| d.id == "DPCS-CAP-005"));
    assert!(diagnostics
        .diagnostics
        .iter()
        .any(|d| d.stage == DiagnosticStage::OrchestratorBinding));
    let cap = capability.expect("capability report retained");
    assert!(cap.missing_mandatory.iter().any(|id| id == "sql.readwrite"));
}

#[test]
fn bind_contract_refuses_invalid_contract() {
    let contract = parse_yaml_file(fixture("invalid/cycle.dpcs.yaml")).unwrap();
    let profile = matching_profile();
    let BindingResult::Err { diagnostics, .. } =
        bind_contract(&contract, &profile, BindingTarget::Dagster)
    else {
        panic!("expected planning refusal");
    };
    assert!(diagnostics
        .diagnostics
        .iter()
        .any(|d| d.id == "DPCS-PLN-001"));
}

#[test]
fn write_bundle_creates_files() {
    let planned = execution_plan();
    let profile = matching_profile();
    let BindingResult::Ok(bundle) = bind(&planned, &profile, BindingTarget::Prefect) else {
        panic!("expected bind");
    };

    let dir = tempfile::tempdir().unwrap();
    write_bundle(&bundle, dir.path()).expect("write");
    let paths: BTreeSet<_> = bundle
        .files
        .iter()
        .map(|f| dir.path().join(&f.relative_path))
        .collect();
    for path in paths {
        assert!(path.is_file(), "missing {}", path.display());
    }
}

#[test]
fn write_bundle_rejects_path_escape() {
    let bundle = BindingBundle {
        target: BindingTarget::Airflow,
        contract_id: "x".into(),
        contract_version: "0.1.0".into(),
        profile_identity: "p".into(),
        files: vec![BindingFile::new("../escape.py", "text/x-python", "pass\n")],
        capability: dpcs::CapabilityReport {
            profile_identity: "p".into(),
            plan_contract_id: None,
            satisfied: vec![],
            missing_mandatory: vec![],
            unsupported_optional: vec![],
            diagnostics: vec![],
        },
    };
    let dir = tempfile::tempdir().unwrap();
    let err = write_bundle(&bundle, dir.path()).unwrap_err();
    assert!(err.diagnostics.iter().any(|d| d.id == "DPCS-BIND-004"));
}

#[test]
fn kubernetes_uses_cronjob_when_scheduled() {
    let planned = execution_plan();
    let profile = matching_profile();
    let BindingResult::Ok(bundle) = bind(&planned, &profile, BindingTarget::Kubernetes) else {
        panic!("expected k8s bind");
    };
    let names: Vec<_> = bundle
        .files
        .iter()
        .map(|f| f.relative_path.as_str())
        .collect();
    assert!(names.contains(&"pipeline-configmap.yaml"));
    assert!(names.contains(&"cronjob.yaml"));
    let cron = bundle
        .files
        .iter()
        .find(|f| f.relative_path == "cronjob.yaml")
        .unwrap();
    assert!(cron.content.contains("kind: CronJob"));
    assert!(cron.content.contains("0 2 * * *"));
    assert!(cron.content.contains("timeZone: \"UTC\""));
}

#[test]
fn kubernetes_uses_init_containers_for_multi_step() {
    let contract = parse_yaml_file(fixture("valid/with_graph_features.dpcs.yaml")).unwrap();
    let planned = try_plan(&contract).expect("plan");
    let profile = matching_profile();
    let BindingResult::Ok(bundle) = bind(&planned, &profile, BindingTarget::Kubernetes) else {
        panic!("expected k8s");
    };
    let job = bundle
        .files
        .iter()
        .find(|f| f.relative_path == "job.yaml")
        .expect("job.yaml");
    assert!(job.content.contains("initContainers:"));
    assert!(
        !job.content
            .contains("containers:\n        - name: ingest\n          image")
            || job.content.contains("initContainers:")
    );
}

#[test]
fn airflow_embeds_schedule_and_contract_refs() {
    let planned = execution_plan();
    let profile = matching_profile();
    let BindingResult::Ok(bundle) = bind(&planned, &profile, BindingTarget::Airflow) else {
        panic!("expected airflow");
    };
    let content = &bundle.files[0].content;
    assert!(content.contains("0 2 * * *"));
    assert!(content.contains("normalize_customer_transform") || content.contains("contractRef"));
    assert!(!content.contains("dag.timezone ="));
}

#[test]
fn temporal_uses_pascal_case_class() {
    let planned = execution_plan();
    let profile = matching_profile();
    let BindingResult::Ok(bundle) = bind(&planned, &profile, BindingTarget::Temporal) else {
        panic!("expected temporal");
    };
    assert!(bundle.files[0]
        .content
        .contains("class ValidExecutionModelWorkflow"));
}
