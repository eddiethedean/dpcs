//! Pipeline Plan types and deterministic planner (SPEC Ch 15).

use serde::{Deserialize, Serialize};

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{
    ContractReference, DependencyGraph, ExecutionRequirements, FailureSemantics, PipelineContract,
    PipelineGraph, PipelineLineage, PipelineStep, QualityGate, SchedulingIntent,
};
use crate::validation;

/// Directed dependency edge captured in a Pipeline Plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanDependencyEdge {
    /// Source step identifier.
    pub from: String,
    /// Destination step identifier.
    pub to: String,
}

/// Canonical intermediate representation produced from a validated contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelinePlan {
    /// Identity of the source pipeline contract.
    pub contract_id: String,
    /// Version of the source pipeline contract.
    pub contract_version: String,
    /// DPCS specification version declared by the source contract.
    pub dpcs_version: String,
    /// Optional independent Pipeline Plan version (SPEC Ch 20).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Resolved pipeline steps.
    pub steps: Vec<PipelineStep>,
    /// Resolved pipeline graph.
    pub graph: PipelineGraph,
    /// Resolved contract references.
    pub contract_references: Vec<ContractReference>,
    /// Dependency edges derived from graph, control flow, and data flow.
    pub dependency_edges: Vec<PlanDependencyEdge>,
    /// Ordered step identifiers in plan order.
    ///
    /// Ordering is a deterministic topological sort over the dependency graph.
    /// Ready nodes are emitted in sorted step-id order (not declaration order).
    pub step_order: Vec<String>,
    /// Preserved execution requirements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionRequirements>,
    /// Preserved scheduling intents.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scheduling: Vec<SchedulingIntent>,
    /// Preserved quality gates.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quality_gates: Vec<QualityGate>,
    /// Preserved failure semantics.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failure_semantics: Vec<FailureSemantics>,
    /// Preserved lineage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage: Option<PipelineLineage>,
}

/// Result of attempting to produce a Pipeline Plan.
#[derive(Debug, Clone, PartialEq)]
pub enum PlanResult {
    /// Plan produced from a successfully validated contract.
    Ok(Box<PipelinePlan>),
    /// Planning refused because the contract is invalid or incomplete.
    Err(ValidationReport),
}

impl PlanResult {
    /// Returns the plan when planning succeeded.
    pub fn plan(self) -> Option<PipelinePlan> {
        match self {
            Self::Ok(plan) => Some(*plan),
            Self::Err(_) => None,
        }
    }

    /// Returns a reference to the plan when planning succeeded.
    pub fn as_plan(&self) -> Option<&PipelinePlan> {
        match self {
            Self::Ok(plan) => Some(plan),
            Self::Err(_) => None,
        }
    }

    /// Returns whether planning succeeded.
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    /// Returns planning/validation diagnostics when planning failed.
    pub fn report(&self) -> Option<&ValidationReport> {
        match self {
            Self::Ok(_) => None,
            Self::Err(report) => Some(report),
        }
    }
}

/// Build a Pipeline Plan from a successfully validated contract.
///
/// When the contract has validation errors, returns [`PlanResult::Err`] with
/// planning-stage diagnostics (`DPCS-PLN-001`) plus the underlying validation
/// findings. Callers SHOULD prefer this over assuming a plan can always be built.
pub fn plan(contract: &PipelineContract) -> PlanResult {
    let report = validation::validate(contract);
    if !report.is_valid() {
        let mut planning_report = ValidationReport::new();
        planning_report.push(
            Diagnostic::planning_error(
                "DPCS-PLN-001",
                categories::PLANNING,
                "pipeline plan requires a successfully validated contract",
            )
            .with_remediation("Resolve validation errors before planning"),
        );
        planning_report.extend(report);
        planning_report.sort_deterministic();
        return PlanResult::Err(planning_report);
    }

    let dependency_graph = DependencyGraph::from_contract(contract);
    // Validated contracts are acyclic; topological_order is infallible here.
    let step_order = dependency_graph
        .topological_order()
        .unwrap_or_else(|_| contract.steps.iter().map(|step| step.id.clone()).collect());

    let dependency_edges = dependency_graph
        .edges()
        .into_iter()
        .map(|(from, to)| PlanDependencyEdge { from, to })
        .collect();

    PlanResult::Ok(Box::new(PipelinePlan {
        contract_id: contract.id.clone(),
        contract_version: contract.version.clone(),
        dpcs_version: contract.dpcs_version.clone(),
        version: None,
        steps: contract.steps.clone(),
        graph: contract.graph.clone(),
        contract_references: contract.contract_references.clone(),
        dependency_edges,
        step_order,
        execution: contract.execution.clone(),
        scheduling: contract.scheduling.clone(),
        quality_gates: contract.quality_gates.clone(),
        failure_semantics: contract.failure_semantics.clone(),
        lineage: contract.lineage.clone(),
    }))
}

/// Convenience wrapper that returns only the plan when planning succeeds.
pub fn try_plan(contract: &PipelineContract) -> Option<PipelinePlan> {
    plan(contract).plan()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_yaml;

    #[test]
    fn refuses_invalid_contract_with_pln_001() {
        let contract = parse_yaml(
            r#"
dpcsVersion: "1.0.0"
id: "test"
version: "0.1.0"
interface:
  inputs: []
  outputs: []
steps:
  - id: ""
    type: "extension:noop"
graph:
  edges: []
"#,
        )
        .unwrap();

        let PlanResult::Err(report) = plan(&contract) else {
            panic!("expected planning refusal");
        };
        assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-PLN-001"));
        assert!(report.diagnostics.iter().any(|d| d.id == "DPCS-COM-004"));
    }

    #[test]
    fn plans_valid_minimal_contract() {
        let contract = parse_yaml(
            r#"
dpcsVersion: "1.0.0"
id: "test"
version: "0.1.0"
interface:
  inputs: []
  outputs: []
steps:
  - id: "a"
    type: "extension:noop"
  - id: "b"
    type: "extension:noop"
graph:
  edges:
    - from: "a"
      to: "b"
"#,
        )
        .unwrap();

        let PlanResult::Ok(plan) = plan(&contract) else {
            panic!("expected successful plan");
        };
        assert_eq!(plan.step_order, vec!["a", "b"]);
        assert_eq!(plan.dependency_edges.len(), 1);
        assert_eq!(plan.dependency_edges[0].from, "a");
        assert_eq!(plan.dependency_edges[0].to, "b");
    }

    #[test]
    fn independent_steps_use_sorted_id_tie_break() {
        let contract = parse_yaml(
            r#"
dpcsVersion: "1.0.0"
id: "test"
version: "0.1.0"
interface:
  inputs: []
  outputs: []
steps:
  - id: "z"
    type: "extension:noop"
  - id: "a"
    type: "extension:noop"
graph:
  edges: []
"#,
        )
        .unwrap();

        let PlanResult::Ok(plan) = plan(&contract) else {
            panic!("expected successful plan");
        };
        assert_eq!(plan.step_order, vec!["a", "z"]);
    }
}
