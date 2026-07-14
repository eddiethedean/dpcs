//! Report module tests.

use dpcs::{
    graph_to_dot, graph_to_mermaid, inspect_to_markdown, inspect_view_from_contract, parse_yaml,
    to_dot, to_mermaid, validation_to_html, validation_to_markdown, GraphEdgeView, GraphView,
    ReportFormat,
};

const MULTI_STEP: &str = r#"
dpcsVersion: "1.0.0"
id: "demo.graph"
version: "0.1.0"
interface:
  inputs:
    - id: "in"
      name: "In"
      contractRef: "c_in"
      purpose: "input"
  outputs:
    - id: "out"
      name: "Out"
      contractRef: "c_out"
      purpose: "output"
contractReferences:
  - id: "c_in"
    type: "odcs"
    location: "in.yaml"
  - id: "c_out"
    type: "odcs"
    location: "out.yaml"
  - id: "t1"
    type: "dtcs"
    location: "t1.yaml"
  - id: "t2"
    type: "dtcs"
    location: "t2.yaml"
steps:
  - id: "a"
    type: "dtcs:transform"
    contractRef: "t1"
    inputs:
      - id: "in"
    outputs:
      - id: "mid"
  - id: "b"
    type: "dtcs:transform"
    contractRef: "t2"
    inputs:
      - id: "mid"
    outputs:
      - id: "out"
graph:
  entryPoints: ["a"]
  exitPoints: ["b"]
  edges:
    - from: "a"
      to: "b"
      kind: "sequence"
dataFlow:
  - from: "interface.inputs.in"
    to: "steps.a.inputs.in"
    dataset: "in"
  - from: "steps.a.outputs.mid"
    to: "steps.b.inputs.mid"
    dataset: "mid"
  - from: "steps.b.outputs.out"
    to: "interface.outputs.out"
    dataset: "out"
"#;

#[test]
fn report_format_parses() {
    assert_eq!(ReportFormat::parse("md").unwrap(), ReportFormat::Markdown);
    assert_eq!(ReportFormat::parse("DOT").unwrap(), ReportFormat::Dot);
    assert!(ReportFormat::parse("xml").is_err());
}

#[test]
fn mermaid_and_dot_include_steps() {
    let contract = parse_yaml(MULTI_STEP).unwrap();
    let mermaid = to_mermaid(&contract);
    assert!(mermaid.contains("flowchart LR"));
    assert!(mermaid.contains("a") && mermaid.contains("b"));
    assert!(mermaid.contains("sequence"));
    let dot = to_dot(&contract);
    assert!(dot.contains("digraph"));
    assert!(dot.contains("a -> b"));
}

#[test]
fn graph_helpers_match_view_exports() {
    let view = GraphView {
        contract_id: Some("x".into()),
        entry_points: vec!["a".into()],
        exit_points: vec!["b".into()],
        step_ids: vec!["a".into(), "b".into()],
        edges: vec![GraphEdgeView {
            from: "a".into(),
            to: "b".into(),
            kind: None,
        }],
        step_order: Some(vec!["a".into(), "b".into()]),
        planning_refused: false,
    };
    assert!(graph_to_mermaid(&view).contains("a --> b"));
    assert!(graph_to_dot(&view).contains("a -> b"));
}

#[test]
fn inspect_markdown_and_validation_html() {
    let contract = parse_yaml(MULTI_STEP).unwrap();
    let view = inspect_view_from_contract(&contract);
    let md = inspect_to_markdown(&view);
    assert!(md.contains("# Pipeline Inspect"));
    assert!(md.contains("demo.graph"));
    let report = dpcs::validate(&contract);
    let html = validation_to_html(&report);
    assert!(html.contains("<!DOCTYPE html>"));
    let md_val = validation_to_markdown(&report);
    assert!(md_val.contains("# Validation Report"));
}
