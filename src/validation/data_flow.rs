//! Data-flow validation phase.

use std::collections::BTreeSet;

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::PipelineContract;

/// Validate data-flow endpoints against known addressable objects.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();
    let endpoints = known_endpoints(contract);

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

        if !endpoint_known(&endpoints, &flow.from) {
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

        if !endpoint_known(&endpoints, &flow.to) {
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

fn known_endpoints(contract: &PipelineContract) -> BTreeSet<String> {
    let mut endpoints = BTreeSet::new();

    for port in &contract.interface.inputs {
        endpoints.insert(format!("interface.inputs.{}", port.id));
    }
    for port in &contract.interface.outputs {
        endpoints.insert(format!("interface.outputs.{}", port.id));
    }
    for step in &contract.steps {
        endpoints.insert(format!("steps.{}", step.id));
        if step.inputs.is_empty() {
            // Steps may omit explicit ports; allow conventional input/output paths.
            endpoints.insert(format!("steps.{}.inputs", step.id));
            endpoints.insert(format!("steps.{}.outputs", step.id));
        } else {
            for port in &step.inputs {
                endpoints.insert(format!("steps.{}.inputs.{}", step.id, port.id));
            }
            for port in &step.outputs {
                endpoints.insert(format!("steps.{}.outputs.{}", step.id, port.id));
            }
        }
    }

    endpoints
}

fn endpoint_known(endpoints: &BTreeSet<String>, endpoint: &str) -> bool {
    if endpoints.contains(endpoint) {
        return true;
    }

    // Allow steps.<id>.inputs.<name> when the step exists but ports are implicit.
    if let Some(rest) = endpoint.strip_prefix("steps.") {
        let mut parts = rest.split('.');
        if let Some(step_id) = parts.next() {
            let prefix = format!("steps.{step_id}");
            if endpoints.contains(&prefix) {
                return matches!(parts.next(), Some("inputs") | Some("outputs"));
            }
        }
    }

    false
}
