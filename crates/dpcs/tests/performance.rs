//! Equivalence and incremental-cache tests for ROADMAP 0.12.

use dpcs::{
    synth, validate, validate_cached, validate_sequential, AnalysisContext, DependencyGraph,
    SchedulingIntent, SchedulingMode, ValidationCache,
};

fn diagnostic_ids(report: &dpcs::ValidationReport) -> Vec<(String, Option<String>, String)> {
    report
        .diagnostics
        .iter()
        .map(|d| (d.id.clone(), d.object_ref.clone(), d.message.clone()))
        .collect()
}

#[test]
fn validate_matches_sequential_on_examples_and_synth() {
    let paths = [
        "examples/minimal.dpcs.yaml",
        "examples/with_execution.dpcs.yaml",
        "examples/with_security_governance.dpcs.yaml",
    ];
    for path in paths {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let contract = dpcs::parse_yaml_file(root.join(path)).expect(path);
        let a = diagnostic_ids(&validate(&contract));
        let b = diagnostic_ids(&validate_sequential(&contract));
        assert_eq!(a, b, "mismatch for {path}");
    }

    for contract in [
        synth::linear_pipeline(50),
        synth::dag_pipeline(16),
        synth::dense_pipeline(80, 2),
        synth::wide_data_flow(40),
    ] {
        let a = diagnostic_ids(&validate(&contract));
        let b = diagnostic_ids(&validate_sequential(&contract));
        assert_eq!(a, b, "mismatch for {}", contract.id);
    }
}

#[test]
fn cached_validation_reuses_phases_on_unrelated_mutation() {
    let mut contract = synth::linear_pipeline(20);
    let mut cache = ValidationCache::new();

    let first = validate_cached(&contract, &mut cache);
    assert!(cache.stats().phases_run > 0);
    assert_eq!(cache.stats().phases_reused, 0);

    let cold_ids = diagnostic_ids(&first);
    assert_eq!(cold_ids, diagnostic_ids(&validate(&contract)));

    // Re-validate identical contract: all phases reused.
    let second = validate_cached(&contract, &mut cache);
    assert_eq!(diagnostic_ids(&second), cold_ids);
    assert_eq!(cache.stats().phases_run, 0);
    assert!(cache.stats().phases_reused >= 10);

    // Mutate only scheduling: scheduling (+ possibly document-adjacent) dirty.
    contract.scheduling.push(SchedulingIntent {
        id: Some("nightly".into()),
        mode: SchedulingMode::Scheduled,
        cron: Some("0 0 * * *".into()),
        timezone: None,
        frequency: None,
        windows: Vec::new(),
        blackouts: Vec::new(),
        deadlines: Vec::new(),
        events: Vec::new(),
        constraints: None,
        policies: Vec::new(),
        extensions: Default::default(),
    });
    let third = validate_cached(&contract, &mut cache);
    assert_eq!(diagnostic_ids(&third), diagnostic_ids(&validate(&contract)));
    assert!(cache.stats().phases_run > 0);
    assert!(cache.stats().phases_reused > 0);
}

#[test]
fn analysis_context_matches_from_contract_graph() {
    let contract = synth::linear_pipeline(100);
    let ctx = AnalysisContext::build(&contract);
    let graph = DependencyGraph::from_contract(&contract);
    assert_eq!(ctx.graph.edges(), graph.edges());
    assert_eq!(
        ctx.graph.topological_order().unwrap(),
        graph.topological_order().unwrap()
    );
}

#[test]
fn scale_validate_linear_200() {
    let contract = synth::linear_pipeline(200);
    let report = validate(&contract);
    assert!(report.is_valid(), "{:?}", report.diagnostics);
}
