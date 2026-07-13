//! Integration tests for graph analysis APIs.

use dpcs::{parse_yaml_file, DependencyGraph, PipelineContract};

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(path)
}

fn load(path: &str) -> PipelineContract {
    parse_yaml_file(fixture(path)).unwrap()
}

#[test]
fn dependency_graph_traversal_on_dag() {
    let contract = load("valid/with_graph_features.dpcs.yaml");
    let graph = DependencyGraph::from_contract(&contract);

    assert_eq!(
        graph.walk_bfs("ingest"),
        vec!["ingest", "transform", "publish"]
    );
    assert_eq!(
        graph.predecessors("transform"),
        ["ingest"].into_iter().collect()
    );
    assert_eq!(
        graph.successors("transform"),
        ["publish"].into_iter().collect()
    );
    assert!(graph.dependencies("publish").contains("ingest"));
    assert!(graph.dependents("ingest").contains("publish"));
}

#[test]
fn topological_order_matches_graph_edges() {
    let contract = load("valid/with_graph_features.dpcs.yaml");
    let graph = DependencyGraph::from_contract(&contract);
    assert_eq!(
        graph.topological_order().unwrap(),
        vec!["ingest", "transform", "publish"]
    );
}

#[test]
fn cycle_fixture_produces_cycle_error() {
    let contract = load("invalid/cycle.dpcs.yaml");
    let graph = DependencyGraph::from_contract(&contract);
    assert!(graph.has_cycle());
    assert!(graph.topological_order().is_err());
}

#[test]
fn graph_features_round_trip() {
    let original = load("valid/with_graph_features.dpcs.yaml");
    assert_eq!(original.graph.entry_points, vec!["ingest"]);
    assert_eq!(original.graph.exit_points, vec!["publish"]);
    assert!(original.graph.metadata.is_some());
    assert_eq!(
        original.data_flow[0].contract_ref.as_deref(),
        Some("raw_contract")
    );

    let yaml = original.to_yaml_str().unwrap();
    let round_tripped = PipelineContract::from_yaml_str(&yaml).unwrap();
    assert_eq!(original, round_tripped);
}

#[test]
fn topo_order_is_stable_when_yaml_keys_reordered() {
    let yaml_a = r#"
dpcsVersion: "1.0.0"
id: "order.test"
version: "0.1.0"
interface:
  inputs: []
  outputs: []
graph:
  edges:
    - from: "a"
      to: "c"
    - from: "b"
      to: "c"
steps:
  - id: "a"
    type: "dtcs:transform"
  - id: "b"
    type: "dtcs:transform"
  - id: "c"
    type: "dtcs:transform"
"#;
    let yaml_b = r#"
steps:
  - id: "c"
    type: "dtcs:transform"
  - id: "b"
    type: "dtcs:transform"
  - id: "a"
    type: "dtcs:transform"
graph:
  edges:
    - from: "b"
      to: "c"
    - from: "a"
      to: "c"
dpcsVersion: "1.0.0"
id: "order.test"
version: "0.1.0"
interface:
  inputs: []
  outputs: []
"#;

    let graph_a = DependencyGraph::from_contract(&dpcs::parse_yaml(yaml_a).unwrap());
    let graph_b = DependencyGraph::from_contract(&dpcs::parse_yaml(yaml_b).unwrap());
    assert_eq!(
        graph_a.topological_order().unwrap(),
        graph_b.topological_order().unwrap()
    );
}
