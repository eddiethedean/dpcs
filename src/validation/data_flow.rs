//! Data-flow validation phase.

use std::collections::{BTreeMap, BTreeSet};

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{PipelineContract, PipelineStep};

/// Validate data-flow endpoints against known addressable objects.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();
    let endpoints = known_endpoints(contract);
    let steps_by_id = steps_by_id(contract);

    for (index, flow) in contract.data_flow.iter().enumerate() {
        let object_ref = format!("dataFlow[{index}]");

        if flow.from.trim().is_empty() || flow.to.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-DF-001",
                    categories::DATA_FLOW,
                    "data flow endpoints must not be empty",
                )
                .with_object_ref(object_ref.clone()),
            );
            continue;
        }

        if !endpoint_known(&endpoints, &steps_by_id, &flow.from) {
            report.push(
                Diagnostic::error(
                    "DPCS-DF-002",
                    categories::DATA_FLOW,
                    format!("data flow references unknown endpoint `{}`", flow.from),
                )
                .with_object_ref(object_ref.clone())
                .with_remediation(
                    "Use interface.inputs.<id>, interface.outputs.<id>, or steps.<id>.inputs|outputs.<id>",
                ),
            );
        }

        if !endpoint_known(&endpoints, &steps_by_id, &flow.to) {
            report.push(
                Diagnostic::error(
                    "DPCS-DF-003",
                    categories::DATA_FLOW,
                    format!("data flow references unknown endpoint `{}`", flow.to),
                )
                .with_object_ref(object_ref),
            );
        }
    }

    report
}

fn steps_by_id(contract: &PipelineContract) -> BTreeMap<&str, &PipelineStep> {
    contract
        .steps
        .iter()
        .map(|step| (step.id.as_str(), step))
        .collect()
}

fn known_endpoints(contract: &PipelineContract) -> BTreeSet<String> {
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
            // Steps may omit explicit ports; allow conventional bag endpoints.
            endpoints.insert(format!("steps.{}.inputs", step.id));
            endpoints.insert(format!("steps.{}.outputs", step.id));
        }
    }

    endpoints
}

fn endpoint_known(
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

    // Explicit ports require exact matches inserted into `endpoints`.
    if !step.inputs.is_empty() || !step.outputs.is_empty() {
        return false;
    }

    // Implicit step ports allow exactly one segment after inputs|outputs.
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
