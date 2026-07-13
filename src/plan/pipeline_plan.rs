//! Pipeline Plan types and stub planner.

use serde::{Deserialize, Serialize};

use crate::model::{DependencyGraph, PipelineContract};

/// Canonical intermediate representation produced from a validated contract.
///
/// This is intentionally minimal until roadmap 0.6.0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelinePlan {
    /// Identity of the source pipeline contract.
    pub contract_id: String,
    /// Version of the source pipeline contract.
    pub contract_version: String,
    /// Ordered step identifiers in plan order.
    pub step_order: Vec<String>,
}

/// Build a lightweight plan skeleton from a contract.
///
/// When the dependency graph is acyclic, `step_order` follows a topological
/// ordering. Otherwise declaration order is preserved. Callers SHOULD validate
/// the contract first.
pub fn plan(contract: &PipelineContract) -> PipelinePlan {
    let step_order = match DependencyGraph::from_contract(contract).topological_order() {
        Ok(order) if !order.is_empty() => order,
        _ => contract.steps.iter().map(|step| step.id.clone()).collect(),
    };

    PipelinePlan {
        contract_id: contract.id.clone(),
        contract_version: contract.version.clone(),
        step_order,
    }
}
