//! Shared analysis indexes built once per validate/plan/report pass.

use std::collections::{BTreeMap, BTreeSet};

use super::{known_data_flow_endpoints, DependencyGraph, PipelineContract, PipelineStep};

/// Precomputed indexes for graph analysis and validation hot paths.
///
/// Build once with [`AnalysisContext::build`] and reuse across validation phases,
/// planning, and report views to avoid repeated endpoint / step-id rebuilds.
#[derive(Debug, Clone)]
pub struct AnalysisContext<'a> {
    /// Source contract.
    pub contract: &'a PipelineContract,
    /// Declared non-empty step identifiers.
    pub step_ids: BTreeSet<&'a str>,
    /// Steps keyed by identifier (includes empty ids if present).
    pub steps_by_id: BTreeMap<&'a str, &'a PipelineStep>,
    /// Declared data-flow endpoint paths.
    pub known_endpoints: BTreeSet<String>,
    /// Dependency graph derived from graph edges, control flow, and data flow.
    pub graph: DependencyGraph,
}

impl<'a> AnalysisContext<'a> {
    /// Build analysis indexes for `contract`.
    pub fn build(contract: &'a PipelineContract) -> Self {
        let step_ids = contract.step_ids();
        let steps_by_id = contract
            .steps
            .iter()
            .map(|step| (step.id.as_str(), step))
            .collect();
        let known_endpoints = known_data_flow_endpoints(contract);
        let graph =
            DependencyGraph::from_indexes(contract, &step_ids, &steps_by_id, &known_endpoints);
        Self {
            contract,
            step_ids,
            steps_by_id,
            known_endpoints,
            graph,
        }
    }

    /// Returns whether `endpoint` refers to a declared interface or step port.
    pub fn data_flow_endpoint_known(&self, endpoint: &str) -> bool {
        super::endpoints::endpoint_known_with_indexes(
            &self.known_endpoints,
            &self.steps_by_id,
            endpoint,
        )
    }

    /// Returns a validated inter-step dependency from a data-flow declaration.
    pub fn data_flow_step_dependency(&self, from: &str, to: &str) -> Option<(String, String)> {
        super::endpoints::data_flow_step_dependency_with_indexes(
            &self.step_ids,
            &self.known_endpoints,
            &self.steps_by_id,
            from,
            to,
        )
    }
}
