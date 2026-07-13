//! Pipeline Plan types and stub planner.

use std::collections::BTreeSet;

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
/// ordering and appends any remaining declared steps in declaration order.
/// Otherwise declaration order is preserved. Callers SHOULD validate the
/// contract first.
pub fn plan(contract: &PipelineContract) -> PipelinePlan {
    let declared_order: Vec<String> = contract.steps.iter().map(|step| step.id.clone()).collect();

    let step_order = match DependencyGraph::from_contract(contract).topological_order() {
        Ok(topo) if !topo.is_empty() => {
            let topo_set: BTreeSet<String> = topo.iter().cloned().collect();
            let mut order = topo;
            for step_id in &declared_order {
                if !topo_set.contains(step_id) {
                    order.push(step_id.clone());
                }
            }
            order
        }
        _ => declared_order,
    };

    PipelinePlan {
        contract_id: contract.id.clone(),
        contract_version: contract.version.clone(),
        step_order,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_yaml;

    #[test]
    fn preserves_declaration_order_for_steps_missing_from_topo() {
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
  - id: "a"
    type: "extension:noop"
graph:
  edges: []
"#,
        )
        .unwrap();

        assert_eq!(plan(&contract).step_order, vec!["a", ""]);
    }
}
