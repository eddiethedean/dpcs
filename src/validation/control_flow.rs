//! Control-flow validation phase.

use std::collections::BTreeSet;

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::PipelineContract;

/// Validate control-flow dependencies against known steps.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();
    let step_ids: BTreeSet<&str> = contract.steps.iter().map(|s| s.id.as_str()).collect();

    for (index, flow) in contract.control_flow.iter().enumerate() {
        let object_ref = format!("controlFlow[{index}]");

        if flow.from.trim().is_empty() || flow.to.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-CF-001",
                    categories::CONTROL_FLOW,
                    "control flow endpoints must not be empty",
                )
                .with_object_ref(object_ref.clone()),
            );
            continue;
        }

        if !step_ids.contains(flow.from.as_str()) {
            report.push(
                Diagnostic::error(
                    "DPCS-CF-002",
                    categories::CONTROL_FLOW,
                    format!("control flow references unknown step `{}`", flow.from),
                )
                .with_object_ref(object_ref.clone()),
            );
        }

        if !step_ids.contains(flow.to.as_str()) {
            report.push(
                Diagnostic::error(
                    "DPCS-CF-003",
                    categories::CONTROL_FLOW,
                    format!("control flow references unknown step `{}`", flow.to),
                )
                .with_object_ref(object_ref),
            );
        }
    }

    report
}
