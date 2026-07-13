//! Data-flow validation phase.

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{data_flow_endpoint_known, PipelineContract};

/// Validate data-flow endpoints against known addressable objects.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();

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

        if !data_flow_endpoint_known(contract, &flow.from) {
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

        if !data_flow_endpoint_known(contract, &flow.to) {
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
