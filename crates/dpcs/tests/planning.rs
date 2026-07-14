//! Integration tests for Pipeline Plan generation (ROADMAP 0.6.0).

use dpcs::{parse_yaml_file, plan, try_plan, PlanResult};

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(path)
}

#[test]
fn plans_valid_execution_model_contract() {
    let contract = parse_yaml_file(fixture("valid/with_execution_model.dpcs.yaml")).unwrap();
    let PlanResult::Ok(planned) = plan(&contract) else {
        panic!("expected successful plan");
    };

    assert_eq!(planned.contract_id, "valid.execution.model");
    assert_eq!(planned.step_order, vec!["normalize_customer"]);
    assert!(planned.execution.is_some());
    assert_eq!(planned.scheduling.len(), 1);
    assert_eq!(planned.quality_gates.len(), 1);
    assert_eq!(planned.failure_semantics.len(), 1);
    assert!(planned.lineage.is_some());
    assert_eq!(planned.contract_references.len(), 2);
}

#[test]
fn try_plan_returns_none_for_invalid_contract() {
    let contract = parse_yaml_file(fixture("invalid/cycle.dpcs.yaml")).unwrap();
    assert!(try_plan(&contract).is_none());

    let PlanResult::Err(report) = plan(&contract) else {
        panic!("expected planning refusal");
    };
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-PLN-001"));
    assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-GRP-004"));
    assert!(report
        .diagnostics
        .iter()
        .any(|d| d.stage.to_string() == "planning" && d.id == "DPCS-PLN-001"));
}

#[test]
fn plan_preserves_execution_intents() {
    let contract = parse_yaml_file(fixture("valid/with_execution_model.dpcs.yaml")).unwrap();
    let planned = try_plan(&contract).expect("plan");
    let execution = planned.execution.expect("execution");
    assert!(execution
        .required_capabilities
        .iter()
        .any(|capability| capability == "batch.compute"));
    assert_eq!(planned.scheduling[0].cron.as_deref(), Some("0 2 * * *"));
    assert_eq!(planned.quality_gates[0].id, "pre_run_schema");
    assert_eq!(planned.failure_semantics[0].id, "normalize_retry");
}
