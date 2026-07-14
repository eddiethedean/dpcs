//! Pipeline Plan types and deterministic planner (SPEC Ch 15).

use serde::{Deserialize, Serialize};

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{
    AnalysisContext, ContractReference, DependencyGraph, ExecutionRequirements, FailureSemantics,
    PipelineContract, PipelineGraph, PipelineLineage, PipelineStep, QualityGate, SchedulingIntent,
};
use crate::resolve::{
    apply_nested_provenance, resolve_contract_references, stamp_nested_parents, NestedPipeline,
    ResolveOptions,
};
use crate::validation;

/// Directed dependency edge captured in a Pipeline Plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PlanDependencyEdge {
    /// Source step identifier.
    pub from: String,
    /// Destination step identifier.
    pub to: String,
}

/// Canonical intermediate representation produced from a validated contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
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
    /// Nested Pipeline Contracts loaded during planning (SPEC Ch 5/6).
    ///
    /// Each nested contract retains its own identity and interface; parent–child
    /// links are also recorded on [`Self::lineage`] provenance.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nested: Vec<NestedPlanPipeline>,
}

/// Nested pipeline captured on a [`PipelinePlan`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct NestedPlanPipeline {
    /// Parent step identifier.
    pub parent_step_id: String,
    /// Contract reference id or location used to load the nested contract.
    pub contract_ref: String,
    /// Nested contract identity.
    pub contract_id: String,
    /// Nested contract version.
    pub contract_version: String,
    /// Nested contract DPCS version.
    pub dpcs_version: String,
    /// Nested interface input port identifiers (preserved identity/boundary).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_ports: Vec<String>,
    /// Nested interface output port identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_ports: Vec<String>,
    /// Deterministic nested step order when the nested graph is acyclic.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub step_order: Vec<String>,
    /// Recursively nested children.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<NestedPlanPipeline>,
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
/// Always deep-resolves Contract References before planning (SPEC Ch 7) using
/// [`ResolveOptions::default_for_planning`] (process CWD). Prefer
/// [`plan_with_resolve`] with [`ResolveOptions::from_document_path`] when
/// locations are relative to a contract file.
///
/// When the contract has validation or resolution errors, returns
/// [`PlanResult::Err`] with planning-stage diagnostics (`DPCS-PLN-001`) plus the
/// underlying findings.
pub fn plan(contract: &PipelineContract) -> PlanResult {
    let opts = ResolveOptions::default_for_planning();
    plan_with_resolve(contract, Some(&opts))
}

/// Build a Pipeline Plan, resolving Contract References with `resolve` when
/// provided, otherwise [`ResolveOptions::default_for_planning`].
pub fn plan_with_resolve(
    contract: &PipelineContract,
    resolve: Option<&ResolveOptions>,
) -> PlanResult {
    let ctx = AnalysisContext::build(contract);
    plan_with_context_and_resolve(&ctx, resolve)
}

/// Build a Pipeline Plan using a prebuilt [`AnalysisContext`].
pub fn plan_with_context(ctx: &AnalysisContext<'_>) -> PlanResult {
    let opts = ResolveOptions::default_for_planning();
    plan_with_context_and_resolve(ctx, Some(&opts))
}

/// Build a Pipeline Plan using a prebuilt context and resolve options.
///
/// When `resolve` is `None`, uses [`ResolveOptions::default_for_planning`].
pub fn plan_with_context_and_resolve(
    ctx: &AnalysisContext<'_>,
    resolve: Option<&ResolveOptions>,
) -> PlanResult {
    let contract = ctx.contract;
    let mut report = validation::validate_with_context(ctx);
    let owned_default = ResolveOptions::default_for_planning();
    let opts = resolve.unwrap_or(&owned_default);
    let mut resolution = resolve_contract_references(contract, opts);
    let mut nested_loaded: Vec<NestedPipeline> = Vec::new();
    if !resolution.report.is_valid() {
        report.extend(resolution.report);
    } else {
        stamp_nested_parents(&mut resolution.nested, &contract.id);
        nested_loaded = resolution.nested;
        report.extend(resolution.report);
    }
    if !report.is_valid() {
        let mut planning_report = ValidationReport::new();
        planning_report.push(
            Diagnostic::planning_error(
                "DPCS-PLN-001",
                categories::PLANNING,
                "pipeline plan requires a successfully validated contract",
            )
            .with_remediation("Resolve validation errors before planning")
            .with_related(report.errors().map(|d| d.id.clone())),
        );
        planning_report.extend(report);
        planning_report.sort_deterministic();
        return PlanResult::Err(planning_report);
    }

    let dependency_graph = &ctx.graph;
    let step_order = match dependency_graph.topological_order() {
        Ok(order) => order,
        Err(cycle) => {
            let mut planning_report = ValidationReport::new();
            planning_report.push(
                Diagnostic::planning_error(
                    "DPCS-PLN-002",
                    categories::PLANNING,
                    format!(
                        "pipeline dependency graph contains a cycle: {}",
                        cycle.cycle.join(" -> ")
                    ),
                )
                .with_remediation("Remove cyclic graph / control-flow / data-flow dependencies"),
            );
            planning_report.sort_deterministic();
            return PlanResult::Err(planning_report);
        }
    };

    let dependency_edges = dependency_graph
        .edges()
        .into_iter()
        .map(|(from, to)| PlanDependencyEdge { from, to })
        .collect();

    let mut lineage = contract.lineage.clone();
    apply_nested_provenance(&mut lineage, &contract.id, &nested_loaded);
    let nested = nested_loaded
        .into_iter()
        .map(nested_pipeline_to_plan)
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
        lineage,
        nested,
    }))
}

/// Convenience wrapper that returns only the plan when planning succeeds.
pub fn try_plan(contract: &PipelineContract) -> Option<PipelinePlan> {
    plan(contract).plan()
}

fn nested_pipeline_to_plan(n: NestedPipeline) -> NestedPlanPipeline {
    let step_order = DependencyGraph::from_contract(&n.contract)
        .topological_order()
        .unwrap_or_else(|_| {
            n.contract
                .steps
                .iter()
                .map(|step| step.id.clone())
                .collect()
        });
    NestedPlanPipeline {
        parent_step_id: n.parent_step_id,
        contract_ref: n.contract_ref,
        contract_id: n.contract.id.clone(),
        contract_version: n.contract.version.clone(),
        dpcs_version: n.contract.dpcs_version.clone(),
        input_ports: n
            .contract
            .interface
            .inputs
            .iter()
            .map(|port| port.id.clone())
            .collect(),
        output_ports: n
            .contract
            .interface
            .outputs
            .iter()
            .map(|port| port.id.clone())
            .collect(),
        step_order,
        children: n
            .children
            .into_iter()
            .map(nested_pipeline_to_plan)
            .collect(),
    }
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
