//! Graph validation phase.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::PipelineContract;

/// Validate graph edges and detect prohibited cycles.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();
    let step_ids: BTreeSet<&str> = contract.steps.iter().map(|s| s.id.as_str()).collect();

    let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for step in &contract.steps {
        adjacency.entry(step.id.as_str()).or_default();
    }

    for (index, edge) in contract.graph.edges.iter().enumerate() {
        let object_ref = format!("graph.edges[{index}]");

        if edge.from.trim().is_empty() || edge.to.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-GRP-001",
                    categories::GRAPH,
                    "graph edge endpoints must not be empty",
                )
                .with_object_ref(object_ref.clone()),
            );
            continue;
        }

        if !step_ids.contains(edge.from.as_str()) {
            report.push(
                Diagnostic::error(
                    "DPCS-GRP-002",
                    categories::GRAPH,
                    format!("graph edge references unknown step `{}`", edge.from),
                )
                .with_object_ref(object_ref.clone())
                .with_remediation("Ensure edge endpoints refer to declared step identifiers"),
            );
        }

        if !step_ids.contains(edge.to.as_str()) {
            report.push(
                Diagnostic::error(
                    "DPCS-GRP-003",
                    categories::GRAPH,
                    format!("graph edge references unknown step `{}`", edge.to),
                )
                .with_object_ref(object_ref)
                .with_remediation("Ensure edge endpoints refer to declared step identifiers"),
            );
        }

        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
        adjacency.entry(edge.to.as_str()).or_default();
    }

    if let Some(cycle_node) = find_cycle(&adjacency) {
        report.push(
            Diagnostic::error(
                "DPCS-GRP-004",
                categories::GRAPH,
                format!("pipeline graph contains a prohibited cycle involving `{cycle_node}`"),
            )
            .with_object_ref("graph")
            .with_remediation("Remove cyclic dependencies from graph.edges"),
        );
    }

    report
}

fn find_cycle(adjacency: &BTreeMap<&str, Vec<&str>>) -> Option<String> {
    let mut indegree: BTreeMap<&str, usize> = adjacency.keys().map(|k| (*k, 0usize)).collect();
    for targets in adjacency.values() {
        for target in targets {
            *indegree.entry(target).or_default() += 1;
        }
    }

    let mut queue: VecDeque<&str> = indegree
        .iter()
        .filter_map(|(node, degree)| (*degree == 0).then_some(*node))
        .collect();

    let mut visited = 0usize;
    while let Some(node) = queue.pop_front() {
        visited += 1;
        if let Some(targets) = adjacency.get(node) {
            for target in targets {
                if let Some(degree) = indegree.get_mut(target) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(target);
                    }
                }
            }
        }
    }

    if visited == adjacency.len() {
        None
    } else {
        indegree
            .into_iter()
            .find_map(|(node, degree)| (degree > 0).then(|| node.to_string()))
    }
}
