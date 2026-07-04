//! Pipeline Plan types and stub planner.

use serde::{Deserialize, Serialize};

use crate::model::PipelineContract;

/// Canonical intermediate representation produced from a validated contract.
///
/// This is intentionally minimal in 0.1.0.
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
/// The current implementation preserves declaration order and does not perform
/// topological sorting. Callers SHOULD validate the contract first.
pub fn plan(contract: &PipelineContract) -> PipelinePlan {
    PipelinePlan {
        contract_id: contract.id.clone(),
        contract_version: contract.version.clone(),
        step_order: contract.steps.iter().map(|s| s.id.clone()).collect(),
    }
}
