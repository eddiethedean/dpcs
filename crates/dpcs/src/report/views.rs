//! Stable inspect / graph view models for CLI, reports, and TUI.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::model::{data_flow_step_dependency, DependencyGraph, PipelineContract};
use crate::plan::{plan, PlanResult};
use crate::validation::validate;

/// Summary view of a pipeline contract for inspect / reports / TUI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectView {
    /// Contract id.
    pub id: String,
    /// Optional display name.
    pub name: Option<String>,
    /// Contract version.
    pub version: String,
    /// Declared DPCS specification version.
    pub dpcs_version: String,
    /// Number of steps.
    pub step_count: usize,
    /// Number of graph edges.
    pub edge_count: usize,
    /// Interface input count.
    pub input_count: usize,
    /// Interface output count.
    pub output_count: usize,
    /// Contract reference count.
    pub contract_reference_count: usize,
    /// Data-flow edge count.
    pub data_flow_count: usize,
    /// Control-flow edge count.
    pub control_flow_count: usize,
    /// Scheduling intent count.
    pub scheduling_count: usize,
    /// Quality gate count.
    pub quality_gate_count: usize,
    /// Failure semantics count.
    pub failure_semantics_count: usize,
    /// Whether execution requirements are present.
    pub has_execution: bool,
    /// Whether lineage is present.
    pub has_lineage: bool,
    /// Whether structural validation currently reports no errors.
    pub valid: bool,
    /// Whether planning was refused.
    pub planning_refused: bool,
    /// Planned step order when planning succeeds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_order: Option<Vec<String>>,
    /// Step ids (stable list for TUI / reports).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub step_ids: Vec<String>,
}

/// Graph topology view for exports and TUI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphView {
    /// Contract id when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<String>,
    /// Declared entry points.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entry_points: Vec<String>,
    /// Declared exit points.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exit_points: Vec<String>,
    /// Step identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub step_ids: Vec<String>,
    /// Graph edges (declared graph edges plus control/data-flow step deps).
    pub edges: Vec<GraphEdgeView>,
    /// Planned step order when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_order: Option<Vec<String>>,
    /// Whether planning was refused.
    pub planning_refused: bool,
}

/// A single directed edge in a [`GraphView`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdgeView {
    /// Source node.
    pub from: String,
    /// Destination node.
    pub to: String,
    /// Optional edge kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// Build an [`InspectView`] from a contract.
pub fn inspect_view_from_contract(contract: &PipelineContract) -> InspectView {
    let valid = validate(contract).is_valid();
    let planned = match plan(contract) {
        PlanResult::Ok(plan) => Some(plan.step_order.clone()),
        PlanResult::Err(_) => None,
    };
    InspectView {
        id: contract.id.clone(),
        name: contract.name.clone(),
        version: contract.version.clone(),
        dpcs_version: contract.dpcs_version.clone(),
        step_count: contract.steps.len(),
        edge_count: contract.graph.edges.len(),
        input_count: contract.interface.inputs.len(),
        output_count: contract.interface.outputs.len(),
        contract_reference_count: contract.contract_references.len(),
        data_flow_count: contract.data_flow.len(),
        control_flow_count: contract.control_flow.len(),
        scheduling_count: contract.scheduling.len(),
        quality_gate_count: contract.quality_gates.len(),
        failure_semantics_count: contract.failure_semantics.len(),
        has_execution: contract.execution.is_some(),
        has_lineage: contract.lineage.is_some(),
        valid,
        planning_refused: planned.is_none(),
        step_order: planned,
        step_ids: contract.steps.iter().map(|s| s.id.clone()).collect(),
    }
}

/// Build a [`GraphView`] from a contract.
///
/// Edges include declared `graph.edges` plus step-to-step dependencies implied by
/// control flow and data flow (same sources as [`DependencyGraph`]).
pub fn graph_view_from_contract(contract: &PipelineContract) -> GraphView {
    let (step_order, planning_refused) = match plan(contract) {
        PlanResult::Ok(plan) => (Some(plan.step_order.clone()), false),
        PlanResult::Err(_) => (None, true),
    };
    GraphView {
        contract_id: Some(contract.id.clone()),
        entry_points: contract.graph.entry_points.clone(),
        exit_points: contract.graph.exit_points.clone(),
        step_ids: contract.steps.iter().map(|s| s.id.clone()).collect(),
        edges: collect_graph_edges(contract),
        step_order,
        planning_refused,
    }
}

fn collect_graph_edges(contract: &PipelineContract) -> Vec<GraphEdgeView> {
    let mut edges = Vec::new();
    let mut seen: BTreeSet<(String, String, Option<String>)> = BTreeSet::new();
    let mut pairs: BTreeSet<(String, String)> = BTreeSet::new();

    let push_edge = |edges: &mut Vec<GraphEdgeView>,
                     seen: &mut BTreeSet<(String, String, Option<String>)>,
                     pairs: &mut BTreeSet<(String, String)>,
                     from: String,
                     to: String,
                     kind: Option<String>| {
        if from.trim().is_empty() || to.trim().is_empty() {
            return;
        }
        let pair = (from.clone(), to.clone());
        let key = (from.clone(), to.clone(), kind.clone());
        if seen.insert(key) {
            pairs.insert(pair);
            edges.push(GraphEdgeView { from, to, kind });
        }
    };

    for e in &contract.graph.edges {
        push_edge(
            &mut edges,
            &mut seen,
            &mut pairs,
            e.from.clone(),
            e.to.clone(),
            e.kind.clone(),
        );
    }
    for flow in &contract.control_flow {
        let kind = flow
            .kind
            .clone()
            .filter(|k| !k.is_empty())
            .or_else(|| Some("controlFlow".to_owned()));
        push_edge(
            &mut edges,
            &mut seen,
            &mut pairs,
            flow.from.clone(),
            flow.to.clone(),
            kind,
        );
    }
    for flow in &contract.data_flow {
        if let Some((from_step, to_step)) =
            data_flow_step_dependency(contract, &flow.from, &flow.to)
        {
            push_edge(
                &mut edges,
                &mut seen,
                &mut pairs,
                from_step,
                to_step,
                Some("dataFlow".to_owned()),
            );
        }
    }

    for (from, to) in DependencyGraph::from_contract(contract).edges() {
        if pairs.contains(&(from.clone(), to.clone())) {
            continue;
        }
        push_edge(&mut edges, &mut seen, &mut pairs, from, to, None);
    }

    edges
}
