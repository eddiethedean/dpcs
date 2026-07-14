//! Shared data-flow endpoint resolution for validation and graph analysis.

use std::collections::{BTreeMap, BTreeSet};

use super::{PipelineContract, PipelineStep};

/// Role of a data-flow endpoint in SPEC Ch 8 source/destination rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointRole {
    /// `interface.inputs.*` — valid flow source only.
    InterfaceInput,
    /// `interface.outputs.*` — valid flow destination only.
    InterfaceOutput,
    /// `steps.<id>.inputs.*` — valid flow destination only.
    StepInput,
    /// `steps.<id>.outputs.*` — valid flow source only.
    StepOutput,
}

/// Builds the set of known data-flow endpoint paths for a contract.
pub fn known_data_flow_endpoints(contract: &PipelineContract) -> BTreeSet<String> {
    let mut endpoints = BTreeSet::new();

    for port in &contract.interface.inputs {
        if !port.id.trim().is_empty() {
            endpoints.insert(format!("interface.inputs.{}", port.id));
        }
    }
    for port in &contract.interface.outputs {
        if !port.id.trim().is_empty() {
            endpoints.insert(format!("interface.outputs.{}", port.id));
        }
    }
    for step in &contract.steps {
        if step.id.trim().is_empty() {
            continue;
        }
        if !step.inputs.is_empty() || !step.outputs.is_empty() {
            for port in &step.inputs {
                if !port.id.trim().is_empty() {
                    endpoints.insert(format!("steps.{}.inputs.{}", step.id, port.id));
                }
            }
            for port in &step.outputs {
                if !port.id.trim().is_empty() {
                    endpoints.insert(format!("steps.{}.outputs.{}", step.id, port.id));
                }
            }
        } else {
            endpoints.insert(format!("steps.{}.inputs", step.id));
            endpoints.insert(format!("steps.{}.outputs", step.id));
        }
    }

    endpoints
}

/// Returns whether `endpoint` refers to a declared interface or step port.
pub fn data_flow_endpoint_known(contract: &PipelineContract, endpoint: &str) -> bool {
    let endpoints = known_data_flow_endpoints(contract);
    let steps_by_id = steps_by_id(contract);
    endpoint_known_with_indexes(&endpoints, &steps_by_id, endpoint)
}

/// Classifies a known-or-bag endpoint into a SPEC source/destination role.
pub fn endpoint_role(endpoint: &str) -> Option<EndpointRole> {
    if let Some(rest) = endpoint.strip_prefix("interface.inputs.") {
        return (!rest.is_empty()).then_some(EndpointRole::InterfaceInput);
    }
    if let Some(rest) = endpoint.strip_prefix("interface.outputs.") {
        return (!rest.is_empty()).then_some(EndpointRole::InterfaceOutput);
    }
    if endpoint == "interface.inputs" || endpoint == "interface.outputs" {
        return None;
    }

    let rest = endpoint.strip_prefix("steps.")?;
    let mut parts = rest.split('.');
    let step_id = parts.next()?;
    if step_id.is_empty() {
        return None;
    }
    let direction = parts.next()?;
    match direction {
        "inputs" => Some(EndpointRole::StepInput),
        "outputs" => Some(EndpointRole::StepOutput),
        _ => None,
    }
}

/// Returns whether an endpoint is a valid data-flow source.
pub fn is_valid_flow_source(endpoint: &str) -> bool {
    matches!(
        endpoint_role(endpoint),
        Some(EndpointRole::InterfaceInput | EndpointRole::StepOutput)
    )
}

/// Returns whether an endpoint is a valid data-flow destination.
pub fn is_valid_flow_destination(endpoint: &str) -> bool {
    matches!(
        endpoint_role(endpoint),
        Some(EndpointRole::InterfaceOutput | EndpointRole::StepInput)
    )
}

/// Returns a validated inter-step dependency from a data-flow declaration.
///
/// Returns `None` when endpoints are invalid, role-illegal, refer to the same
/// step, or do not resolve to step identifiers.
pub fn data_flow_step_dependency(
    contract: &PipelineContract,
    from: &str,
    to: &str,
) -> Option<(String, String)> {
    let endpoints = known_data_flow_endpoints(contract);
    let steps_by_id = steps_by_id(contract);
    let step_ids = contract.step_ids();
    data_flow_step_dependency_with_indexes(&step_ids, &endpoints, &steps_by_id, from, to)
}

/// Endpoint lookup using precomputed indexes (hot-path form).
pub fn endpoint_known_with_indexes(
    endpoints: &BTreeSet<String>,
    steps_by_id: &BTreeMap<&str, &PipelineStep>,
    endpoint: &str,
) -> bool {
    if endpoints.contains(endpoint) {
        return true;
    }

    let Some(rest) = endpoint.strip_prefix("steps.") else {
        return false;
    };
    let mut parts = rest.split('.');
    let Some(step_id) = parts.next() else {
        return false;
    };
    let Some(step) = steps_by_id.get(step_id) else {
        return false;
    };

    if !step.inputs.is_empty() || !step.outputs.is_empty() {
        return false;
    }

    let Some(direction) = parts.next() else {
        return false;
    };
    if direction != "inputs" && direction != "outputs" {
        return false;
    }
    let Some(port_name) = parts.next() else {
        return false;
    };
    !port_name.is_empty() && parts.next().is_none()
}

/// Inter-step dependency extraction using precomputed indexes (hot-path form).
pub fn data_flow_step_dependency_with_indexes(
    step_ids: &BTreeSet<&str>,
    endpoints: &BTreeSet<String>,
    steps_by_id: &BTreeMap<&str, &PipelineStep>,
    from: &str,
    to: &str,
) -> Option<(String, String)> {
    if !endpoint_known_with_indexes(endpoints, steps_by_id, from)
        || !endpoint_known_with_indexes(endpoints, steps_by_id, to)
    {
        return None;
    }
    if !is_valid_flow_source(from) || !is_valid_flow_destination(to) {
        return None;
    }

    let from_step = super::step_id_from_endpoint(from)?;
    let to_step = super::step_id_from_endpoint(to)?;
    if from_step == to_step {
        return None;
    }

    if step_ids.contains(from_step.as_str()) && step_ids.contains(to_step.as_str()) {
        Some((from_step, to_step))
    } else {
        None
    }
}

fn steps_by_id(contract: &PipelineContract) -> BTreeMap<&str, &PipelineStep> {
    contract
        .steps
        .iter()
        .map(|step| (step.id.as_str(), step))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_yaml;

    #[test]
    fn rejects_invalid_ports_for_dependency_extraction() {
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
    inputs: [{ id: "in" }]
    outputs: [{ id: "out" }]
graph:
  edges: []
dataFlow:
  - from: "steps.a.inputs.missing"
    to: "steps.a.outputs.out"
"#,
        )
        .unwrap();

        assert!(!data_flow_endpoint_known(
            &contract,
            "steps.a.inputs.missing"
        ));
        assert!(data_flow_step_dependency(
            &contract,
            "steps.a.inputs.missing",
            "steps.a.outputs.out"
        )
        .is_none());
    }

    #[test]
    fn skips_same_step_data_flow_dependencies() {
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
    inputs: [{ id: "in" }]
    outputs: [{ id: "out" }]
graph:
  edges: []
dataFlow:
  - from: "steps.a.outputs.out"
    to: "steps.a.inputs.in"
"#,
        )
        .unwrap();

        assert!(
            data_flow_step_dependency(&contract, "steps.a.outputs.out", "steps.a.inputs.in")
                .is_none()
        );
    }

    #[test]
    fn classifies_endpoint_roles() {
        assert_eq!(
            endpoint_role("interface.inputs.raw"),
            Some(EndpointRole::InterfaceInput)
        );
        assert_eq!(
            endpoint_role("steps.a.outputs.out"),
            Some(EndpointRole::StepOutput)
        );
        assert!(is_valid_flow_source("steps.a.outputs.out"));
        assert!(!is_valid_flow_source("steps.a.inputs.in"));
        assert!(is_valid_flow_destination("steps.a.inputs.in"));
    }
}
