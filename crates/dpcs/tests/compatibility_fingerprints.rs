use dpcs::{compare_contracts, parse_yaml, CompatibilityResult};

fn bare_pipe() -> String {
    r#"
dpcsVersion: "1.0.0"
id: "pipe"
version: "1.0.0"
interface: { inputs: [], outputs: [] }
steps: []
graph: { edges: [] }
"#
    .into()
}

#[test]
fn quality_criteria_change_is_incompatible() {
    let mk = |expr: &str| {
        parse_yaml(&format!(
            "{}qualityGates:\n  - id: q1\n    purpose: check\n    criteria:\n      - expression: \"{expr}\"\n    onSuccess: continue\n    onFailure: fail\n",
            bare_pipe()
        ))
        .unwrap()
    };
    assert!(matches!(
        compare_contracts(&mk("x > 0"), &mk("x > 100")),
        CompatibilityResult::Err { .. }
    ));
}

#[test]
fn failure_responses_change_is_incompatible() {
    let mk = |resp: &str| {
        parse_yaml(&format!(
            "{}failureSemantics:\n  - id: f1\n    scope: {{ kind: pipeline }}\n    triggers: [timeout]\n    responses: [{resp}]\n",
            bare_pipe()
        ))
        .unwrap()
    };
    assert!(matches!(
        compare_contracts(&mk("abort"), &mk("continue")),
        CompatibilityResult::Err { .. }
    ));
}

#[test]
fn scheduling_timezone_change_is_incompatible() {
    let mk = |tz: &str| {
        parse_yaml(&format!(
            "{}scheduling:\n  - mode: scheduled\n    cron: \"0 * * * *\"\n    timezone: \"{tz}\"\n",
            bare_pipe()
        ))
        .unwrap()
    };
    assert!(matches!(
        compare_contracts(&mk("UTC"), &mk("America/New_York")),
        CompatibilityResult::Err { .. }
    ));
}

#[test]
fn lineage_edge_change_is_incompatible() {
    let a = parse_yaml(&format!(
        "{}lineage:\n  datasets:\n    - dataset: d\n      producedBy: s1\n",
        bare_pipe()
    ))
    .unwrap();
    let b = parse_yaml(&format!(
        "{}lineage:\n  datasets:\n    - dataset: d\n      producedBy: s2\n",
        bare_pipe()
    ))
    .unwrap();
    assert!(matches!(
        compare_contracts(&a, &b),
        CompatibilityResult::Err { .. }
    ));
}
