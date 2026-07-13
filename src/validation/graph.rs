//! Graph validation phase.

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{DependencyGraph, PipelineContract};

/// Validate graph edges and detect prohibited cycles.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();
    let step_ids = contract.step_ids();

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
    }

    for duplicate in DependencyGraph::duplicate_edges(contract) {
        report.push(
            Diagnostic::error(
                "DPCS-GRP-005",
                categories::GRAPH,
                format!(
                    "duplicate graph edge from `{}` to `{}`",
                    duplicate.from, duplicate.to
                ),
            )
            .with_object_ref(format!("graph.edges[{}]", duplicate.duplicate_index))
            .with_remediation("Remove duplicate edges from graph.edges"),
        );
    }

    let dependency_graph = DependencyGraph::from_contract(contract);

    if let Some(cycle) = dependency_graph.find_cycle() {
        let cycle_node = cycle.first().cloned().unwrap_or_default();
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

    let unreachable = dependency_graph.unreachable_steps(contract);
    for step_id in unreachable {
        report.push(
            Diagnostic::error(
                "DPCS-GRP-006",
                categories::GRAPH,
                format!("pipeline step `{step_id}` is unreachable from graph entry points"),
            )
            .with_object_ref(format!("steps.{step_id}"))
            .with_remediation("Declare graph.entryPoints or add edges so every step is reachable"),
        );
    }

    report
}
