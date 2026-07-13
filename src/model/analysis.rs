//! Pipeline graph analysis: traversal, cycle detection, and dependency analysis.
//!
//! Builds a directed step dependency graph from explicit graph edges, control flow,
//! and data flow where both endpoints resolve to steps.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::PipelineContract;

/// A directed step dependency graph derived from a Pipeline Contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyGraph {
    nodes: BTreeSet<String>,
    successors: BTreeMap<String, BTreeSet<String>>,
    predecessors: BTreeMap<String, BTreeSet<String>>,
}

/// Error returned when a topological ordering cannot be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleError {
    /// A concrete cycle path when one can be determined.
    pub cycle: Vec<String>,
}

impl std::fmt::Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "pipeline graph contains a cycle: {}",
            self.cycle.join(" -> ")
        )
    }
}

impl std::error::Error for CycleError {}

/// A duplicate explicit graph edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateEdge {
    /// Source step identifier.
    pub from: String,
    /// Destination step identifier.
    pub to: String,
    /// Optional edge kind.
    pub kind: Option<String>,
    /// Index of the first occurrence in `graph.edges`.
    pub first_index: usize,
    /// Index of the duplicate occurrence in `graph.edges`.
    pub duplicate_index: usize,
}

impl DependencyGraph {
    /// Builds a dependency graph from the contract's graph, control flow, and data flow.
    pub fn from_contract(contract: &PipelineContract) -> Self {
        let mut graph = Self::empty();
        let step_ids = contract.step_ids();

        for step in &contract.steps {
            if !step.id.trim().is_empty() {
                graph.ensure_node(&step.id);
            }
        }

        for edge in &contract.graph.edges {
            if edge.from.trim().is_empty() || edge.to.trim().is_empty() {
                continue;
            }
            if step_ids.contains(edge.from.as_str()) && step_ids.contains(edge.to.as_str()) {
                graph.add_edge(&edge.from, &edge.to);
            }
        }

        for flow in &contract.control_flow {
            if flow.from.trim().is_empty() || flow.to.trim().is_empty() {
                continue;
            }
            if step_ids.contains(flow.from.as_str()) && step_ids.contains(flow.to.as_str()) {
                graph.add_edge(&flow.from, &flow.to);
            }
        }

        for flow in &contract.data_flow {
            if flow.from.trim().is_empty() || flow.to.trim().is_empty() {
                continue;
            }
            if let (Some(from_step), Some(to_step)) = (
                step_id_from_endpoint(&flow.from),
                step_id_from_endpoint(&flow.to),
            ) {
                if step_ids.contains(from_step.as_str()) && step_ids.contains(to_step.as_str()) {
                    graph.add_edge(&from_step, &to_step);
                }
            }
        }

        graph
    }

    /// Returns all step identifiers present in the dependency graph.
    pub fn nodes(&self) -> &BTreeSet<String> {
        &self.nodes
    }

    /// Returns direct successor step identifiers for `step_id`.
    pub fn successors(&self, step_id: &str) -> BTreeSet<&str> {
        self.successors
            .get(step_id)
            .map(|set| set.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }

    /// Returns direct predecessor step identifiers for `step_id`.
    pub fn predecessors(&self, step_id: &str) -> BTreeSet<&str> {
        self.predecessors
            .get(step_id)
            .map(|set| set.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }

    /// Returns all transitive dependencies (predecessors) of `step_id`.
    pub fn dependencies(&self, step_id: &str) -> BTreeSet<String> {
        transitive_closure(step_id, &self.predecessors)
    }

    /// Returns all transitive dependents (successors) of `step_id`.
    pub fn dependents(&self, step_id: &str) -> BTreeSet<String> {
        transitive_closure(step_id, &self.successors)
    }

    /// Depth-first traversal from `start`, visiting successors in sorted order.
    pub fn walk_dfs(&self, start: &str) -> Vec<String> {
        let mut visited = BTreeSet::new();
        let mut order = Vec::new();
        self.dfs_visit(start, &mut visited, &mut order);
        order
    }

    /// Breadth-first traversal from `start`, visiting successors in sorted order.
    pub fn walk_bfs(&self, start: &str) -> Vec<String> {
        let mut visited = BTreeSet::new();
        let mut order = Vec::new();
        let mut queue = VecDeque::new();

        if !self.nodes.contains(start) {
            return order;
        }

        queue.push_back(start.to_string());
        visited.insert(start.to_string());

        while let Some(node) = queue.pop_front() {
            order.push(node.clone());
            if let Some(successors) = self.successors.get(&node) {
                for successor in successors {
                    if visited.insert(successor.clone()) {
                        queue.push_back(successor.clone());
                    }
                }
            }
        }

        order
    }

    /// Returns whether the dependency graph contains a cycle.
    pub fn has_cycle(&self) -> bool {
        self.find_cycle().is_some()
    }

    /// Returns a concrete cycle path when one exists.
    pub fn find_cycle(&self) -> Option<Vec<String>> {
        let mut state: BTreeMap<String, u8> =
            self.nodes.iter().map(|node| (node.clone(), 0)).collect();

        for node in self.nodes.iter() {
            if state[node] == 0 {
                let mut path = Vec::new();
                if dfs_cycle(node, &self.successors, &mut state, &mut path) {
                    return Some(path);
                }
            }
        }

        None
    }

    /// Returns a topological ordering of step identifiers, or a cycle error.
    pub fn topological_order(&self) -> Result<Vec<String>, CycleError> {
        let mut indegree: BTreeMap<&str, usize> = self
            .nodes
            .iter()
            .map(|node| (node.as_str(), 0usize))
            .collect();

        for successors in self.successors.values() {
            for target in successors {
                if let Some(degree) = indegree.get_mut(target.as_str()) {
                    *degree += 1;
                }
            }
        }

        let mut queue: VecDeque<&str> = indegree
            .iter()
            .filter_map(|(node, degree)| (*degree == 0).then_some(*node))
            .collect();

        let mut order = Vec::with_capacity(self.nodes.len());
        while let Some(node) = queue.pop_front() {
            order.push(node.to_string());
            if let Some(successors) = self.successors.get(node) {
                for target in successors {
                    if let Some(degree) = indegree.get_mut(target.as_str()) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(target.as_str());
                        }
                    }
                }
            }
        }

        if order.len() == self.nodes.len() {
            Ok(order)
        } else {
            Err(CycleError {
                cycle: self.find_cycle().unwrap_or_default(),
            })
        }
    }

    /// Returns step identifiers not reachable from declared entry points or indegree-zero roots.
    pub fn unreachable_steps(&self, contract: &PipelineContract) -> BTreeSet<String> {
        let declared: BTreeSet<String> = contract
            .graph
            .entry_points
            .iter()
            .filter(|id| !id.trim().is_empty() && self.nodes.contains(id.as_str()))
            .cloned()
            .collect();

        let roots: Vec<String> = if declared.is_empty() {
            self.nodes
                .iter()
                .filter(|node| self.predecessors(node).is_empty())
                .cloned()
                .collect()
        } else {
            declared.into_iter().collect()
        };

        let mut reachable = BTreeSet::new();
        for root in roots {
            for node in self.walk_bfs(&root) {
                reachable.insert(node);
            }
        }

        self.nodes.difference(&reachable).cloned().collect()
    }

    /// Returns duplicate explicit graph edges with identical `(from, to, kind)` tuples.
    pub fn duplicate_edges(contract: &PipelineContract) -> Vec<DuplicateEdge> {
        let mut seen: BTreeMap<(String, String, Option<String>), usize> = BTreeMap::new();
        let mut duplicates = Vec::new();

        for (index, edge) in contract.graph.edges.iter().enumerate() {
            if edge.from.trim().is_empty() || edge.to.trim().is_empty() {
                continue;
            }
            let key = (edge.from.clone(), edge.to.clone(), edge.kind.clone());
            if let Some(first_index) = seen.get(&key) {
                duplicates.push(DuplicateEdge {
                    from: edge.from.clone(),
                    to: edge.to.clone(),
                    kind: edge.kind.clone(),
                    first_index: *first_index,
                    duplicate_index: index,
                });
            } else {
                seen.insert(key, index);
            }
        }

        duplicates
    }

    fn empty() -> Self {
        Self {
            nodes: BTreeSet::new(),
            successors: BTreeMap::new(),
            predecessors: BTreeMap::new(),
        }
    }

    fn ensure_node(&mut self, id: &str) {
        self.nodes.insert(id.to_string());
        self.successors.entry(id.to_string()).or_default();
        self.predecessors.entry(id.to_string()).or_default();
    }

    fn add_edge(&mut self, from: &str, to: &str) {
        self.ensure_node(from);
        self.ensure_node(to);
        self.successors
            .entry(from.to_string())
            .or_default()
            .insert(to.to_string());
        self.predecessors
            .entry(to.to_string())
            .or_default()
            .insert(from.to_string());
    }

    fn dfs_visit(&self, node: &str, visited: &mut BTreeSet<String>, order: &mut Vec<String>) {
        if !visited.insert(node.to_string()) {
            return;
        }
        order.push(node.to_string());
        if let Some(successors) = self.successors.get(node) {
            for successor in successors {
                self.dfs_visit(successor, visited, order);
            }
        }
    }
}

fn transitive_closure(
    start: &str,
    adjacency: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(start.to_string());

    while let Some(node) = queue.pop_front() {
        if let Some(neighbors) = adjacency.get(&node) {
            for neighbor in neighbors {
                if visited.insert(neighbor.clone()) {
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    visited
}

fn dfs_cycle(
    node: &str,
    successors: &BTreeMap<String, BTreeSet<String>>,
    state: &mut BTreeMap<String, u8>,
    path: &mut Vec<String>,
) -> bool {
    state.insert(node.to_string(), 1);
    path.push(node.to_string());

    if let Some(nexts) = successors.get(node) {
        for next in nexts {
            match state.get(next).copied().unwrap_or(0) {
                1 => {
                    path.push(next.clone());
                    return true;
                }
                0 if dfs_cycle(next, successors, state, path) => return true,
                _ => {}
            }
        }
    }

    state.insert(node.to_string(), 2);
    path.pop();
    false
}

/// Extracts a step identifier from a data-flow endpoint when it references a step.
pub fn step_id_from_endpoint(endpoint: &str) -> Option<String> {
    let rest = endpoint.strip_prefix("steps.")?;
    let mut parts = rest.split('.');
    let step_id = parts.next()?;
    if step_id.is_empty() {
        return None;
    }
    let direction = parts.next()?;
    if direction != "inputs" && direction != "outputs" {
        return None;
    }
    Some(step_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_yaml;

    fn contract(yaml: &str) -> PipelineContract {
        parse_yaml(yaml).expect("parse contract")
    }

    #[test]
    fn builds_edges_from_graph_control_and_data_flow() {
        let contract = contract(
            r#"
dpcsVersion: "1.0.0"
id: "test.pipeline"
version: "0.1.0"
interface:
  inputs: []
  outputs: []
graph:
  edges:
    - from: "a"
      to: "b"
steps:
  - id: "a"
    type: "dtcs:transform"
  - id: "b"
    type: "dtcs:transform"
  - id: "c"
    type: "dtcs:transform"
controlFlow:
  - from: "b"
    to: "c"
dataFlow:
  - from: "steps.a.outputs.out"
    to: "steps.c.inputs.in"
    dataset: "ds"
"#,
        );

        let graph = DependencyGraph::from_contract(&contract);
        assert!(graph.successors("a").contains("b"));
        assert!(graph.successors("b").contains("c"));
        assert!(graph.successors("a").contains("c"));
    }

    #[test]
    fn topological_order_is_deterministic() {
        let contract = contract(
            r#"
dpcsVersion: "1.0.0"
id: "test.pipeline"
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
"#,
        );

        let graph = DependencyGraph::from_contract(&contract);
        assert_eq!(
            graph.topological_order().unwrap(),
            vec!["a", "b", "c"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn finds_cycle_path() {
        let contract = contract(
            r#"
dpcsVersion: "1.0.0"
id: "test.pipeline"
version: "0.1.0"
interface:
  inputs: []
  outputs: []
graph:
  edges:
    - from: "a"
      to: "b"
    - from: "b"
      to: "a"
steps:
  - id: "a"
    type: "dtcs:transform"
  - id: "b"
    type: "dtcs:transform"
"#,
        );

        let graph = DependencyGraph::from_contract(&contract);
        assert!(graph.has_cycle());
        assert!(graph.topological_order().is_err());
        let cycle = graph.find_cycle().expect("cycle path");
        assert!(cycle.len() >= 2);
    }

    #[test]
    fn detects_unreachable_steps_from_entry_points() {
        let contract = contract(
            r#"
dpcsVersion: "1.0.0"
id: "test.pipeline"
version: "0.1.0"
interface:
  inputs: []
  outputs: []
graph:
  entryPoints: ["a"]
  edges:
    - from: "a"
      to: "b"
steps:
  - id: "a"
    type: "dtcs:transform"
  - id: "b"
    type: "dtcs:transform"
  - id: "c"
    type: "dtcs:transform"
"#,
        );

        let graph = DependencyGraph::from_contract(&contract);
        let unreachable = graph.unreachable_steps(&contract);
        assert_eq!(unreachable, BTreeSet::from(["c".to_string()]));
    }

    #[test]
    fn detects_duplicate_graph_edges() {
        let contract = contract(
            r#"
dpcsVersion: "1.0.0"
id: "test.pipeline"
version: "0.1.0"
interface:
  inputs: []
  outputs: []
graph:
  edges:
    - from: "a"
      to: "b"
    - from: "a"
      to: "b"
steps:
  - id: "a"
    type: "dtcs:transform"
  - id: "b"
    type: "dtcs:transform"
"#,
        );

        let duplicates = DependencyGraph::duplicate_edges(&contract);
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].first_index, 0);
        assert_eq!(duplicates[0].duplicate_index, 1);
    }
}
