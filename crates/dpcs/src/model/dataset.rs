//! Dataset reachability analysis for data-flow validation.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    data_flow_endpoint_known, endpoint_role, is_valid_flow_destination, is_valid_flow_source,
    step_id_from_endpoint, EndpointRole, PipelineContract,
};

/// Returns dataset identifiers that are not reachable from interface inputs.
///
/// A dataset is sourced when a role-valid flow carrying it starts at an interface
/// input, or leaves a step output after that step's inputs carry sourced datasets.
pub fn unreachable_datasets(contract: &PipelineContract) -> BTreeSet<String> {
    let mut all_datasets = BTreeSet::new();
    let mut flows: Vec<(String, String, String)> = Vec::new();

    for flow in &contract.data_flow {
        let Some(dataset) = flow
            .dataset
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        if flow.from.trim().is_empty()
            || flow.to.trim().is_empty()
            || !data_flow_endpoint_known(contract, &flow.from)
            || !data_flow_endpoint_known(contract, &flow.to)
            || !is_valid_flow_source(&flow.from)
            || !is_valid_flow_destination(&flow.to)
        {
            continue;
        }
        all_datasets.insert(dataset.to_string());
        flows.push((flow.from.clone(), flow.to.clone(), dataset.to_string()));
    }

    if all_datasets.is_empty() {
        return BTreeSet::new();
    }

    let mut incoming: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (_from, to, dataset) in &flows {
        incoming
            .entry(to.clone())
            .or_default()
            .insert(dataset.clone());
    }

    let mut sourced = BTreeSet::new();
    for (from, _to, dataset) in &flows {
        if matches!(endpoint_role(from), Some(EndpointRole::InterfaceInput)) {
            sourced.insert(dataset.clone());
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        for step in &contract.steps {
            if step.id.trim().is_empty() {
                continue;
            }
            if !step_is_sourced(contract, step.id.as_str(), &incoming, &sourced) {
                continue;
            }

            for (from, _to, dataset) in &flows {
                if matches!(endpoint_role(from), Some(EndpointRole::StepOutput))
                    && step_id_from_endpoint(from).as_deref() == Some(step.id.as_str())
                    && sourced.insert(dataset.clone())
                {
                    changed = true;
                }
            }
        }
    }

    all_datasets.difference(&sourced).cloned().collect()
}

fn step_is_sourced(
    contract: &PipelineContract,
    step_id: &str,
    incoming: &BTreeMap<String, BTreeSet<String>>,
    sourced: &BTreeSet<String>,
) -> bool {
    let Some(step) = contract.steps.iter().find(|step| step.id == step_id) else {
        return false;
    };

    let declared_inputs: Vec<String> = if !step.inputs.is_empty() || !step.outputs.is_empty() {
        step.inputs
            .iter()
            .filter(|port| !port.id.trim().is_empty())
            .map(|port| format!("steps.{step_id}.inputs.{}", port.id))
            .collect()
    } else {
        Vec::new()
    };

    if !declared_inputs.is_empty() {
        return declared_inputs.iter().all(|endpoint| {
            incoming
                .get(endpoint)
                .map(|datasets| datasets.iter().any(|dataset| sourced.contains(dataset)))
                .unwrap_or(false)
        });
    }

    // Bag / portless steps: require at least one incoming input flow with a
    // sourced dataset. Never treat "no incoming" as vacuous provenance.
    let mut has_incoming = false;
    let mut all_sourced = true;
    for (endpoint, datasets) in incoming {
        if step_id_from_endpoint(endpoint).as_deref() != Some(step_id) {
            continue;
        }
        if !matches!(endpoint_role(endpoint), Some(EndpointRole::StepInput)) {
            continue;
        }
        has_incoming = true;
        if !datasets.iter().any(|dataset| sourced.contains(dataset)) {
            all_sourced = false;
        }
    }
    has_incoming && all_sourced
}

/// Declared step input and interface output endpoints that lack an incoming data flow.
pub fn unsatisfied_ports(contract: &PipelineContract) -> Vec<(String, String)> {
    let mut targets: BTreeSet<String> = BTreeSet::new();
    for flow in &contract.data_flow {
        if flow.to.trim().is_empty() {
            continue;
        }
        if data_flow_endpoint_known(contract, &flow.to) && is_valid_flow_destination(&flow.to) {
            targets.insert(flow.to.clone());
        }
    }

    let mut missing = Vec::new();

    for port in &contract.interface.outputs {
        if port.id.trim().is_empty() {
            continue;
        }
        let endpoint = format!("interface.outputs.{}", port.id);
        if !targets.contains(&endpoint) {
            missing.push((
                endpoint.clone(),
                format!("interface output `{endpoint}` has no incoming data flow"),
            ));
        }
    }

    for step in &contract.steps {
        if step.id.trim().is_empty() {
            continue;
        }
        for port in &step.inputs {
            if port.id.trim().is_empty() {
                continue;
            }
            let endpoint = format!("steps.{}.inputs.{}", step.id, port.id);
            if !targets.contains(&endpoint) {
                missing.push((
                    endpoint.clone(),
                    format!("step input `{endpoint}` has no incoming data flow"),
                ));
            }
        }
    }

    missing
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_yaml;

    #[test]
    fn flags_datasets_not_fed_from_interface() {
        let contract = parse_yaml(
            r#"
dpcsVersion: "1.0.0"
id: "test"
version: "0.1.0"
interface:
  inputs:
    - id: "raw"
      name: "Raw"
      contractRef: "contracts/raw.odcs.yaml"
      purpose: "raw"
  outputs: []
steps:
  - id: "a"
    type: "extension:noop"
    inputs: [{ id: "in" }]
    outputs: [{ id: "out" }]
  - id: "orphan"
    type: "extension:noop"
    inputs: [{ id: "in" }]
    outputs: [{ id: "out" }]
graph:
  edges: []
dataFlow:
  - from: "interface.inputs.raw"
    to: "steps.a.inputs.in"
    dataset: "raw"
  - from: "steps.orphan.outputs.out"
    to: "steps.orphan.inputs.in"
    dataset: "orphan_ds"
"#,
        )
        .unwrap();

        let unreachable = unreachable_datasets(&contract);
        assert!(unreachable.contains("orphan_ds"));
        assert!(!unreachable.contains("raw"));
    }

    #[test]
    fn does_not_vacuously_source_portless_steps() {
        let contract = parse_yaml(
            r#"
dpcsVersion: "1.0.0"
id: "test"
version: "0.1.0"
interface:
  inputs: []
  outputs: []
steps:
  - id: "ingest"
    type: "extension:ingress"
  - id: "transform"
    type: "dtcs:transform"
graph:
  edges: []
dataFlow:
  - from: "steps.ingest.outputs.raw"
    to: "steps.transform.inputs.raw"
    dataset: "raw"
"#,
        )
        .unwrap();

        let unreachable = unreachable_datasets(&contract);
        assert!(unreachable.contains("raw"));
    }
}
