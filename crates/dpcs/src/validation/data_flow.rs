//! Data-flow validation phase.

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{
    is_valid_flow_destination, is_valid_flow_source, unreachable_datasets, unsatisfied_ports,
    AnalysisContext, PipelineContract,
};

#[allow(dead_code)]
/// Validate data-flow endpoints, roles, dataset identity, wiring, and reachability.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let ctx = AnalysisContext::build(contract);
    validate_with_context(&ctx)
}

/// Validate data flow using a shared analysis context.
pub fn validate_with_context(ctx: &AnalysisContext<'_>) -> ValidationReport {
    let contract = ctx.contract;
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

        let from_known = ctx.data_flow_endpoint_known(&flow.from);
        let to_known = ctx.data_flow_endpoint_known(&flow.to);

        if !from_known {
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
        } else if !is_valid_flow_source(&flow.from) {
            report.push(
                Diagnostic::error(
                    "DPCS-DF-007",
                    categories::DATA_FLOW,
                    format!(
                        "data flow source `{}` must be an interface input or step output",
                        flow.from
                    ),
                )
                .with_object_ref(format!("{object_ref}.from"))
                .with_remediation(
                    "Use interface.inputs.<id> or steps.<id>.outputs.<id> as the source",
                ),
            );
        }

        if !to_known {
            report.push(
                Diagnostic::error(
                    "DPCS-DF-003",
                    categories::DATA_FLOW,
                    format!("data flow references unknown endpoint `{}`", flow.to),
                )
                .with_object_ref(object_ref.clone()),
            );
        } else if !is_valid_flow_destination(&flow.to) {
            report.push(
                Diagnostic::error(
                    "DPCS-DF-008",
                    categories::DATA_FLOW,
                    format!(
                        "data flow destination `{}` must be a step input or interface output",
                        flow.to
                    ),
                )
                .with_object_ref(format!("{object_ref}.to"))
                .with_remediation(
                    "Use steps.<id>.inputs.<id> or interface.outputs.<id> as the destination",
                ),
            );
        }

        if flow
            .dataset
            .as_ref()
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
        {
            report.push(
                Diagnostic::error(
                    "DPCS-DF-004",
                    categories::DATA_FLOW,
                    "data flow must declare a non-empty dataset identity",
                )
                .with_object_ref(format!("{object_ref}.dataset"))
                .with_remediation("Set dataFlow[].dataset to a stable dataset identifier"),
            );
        }
    }

    for (endpoint, message) in unsatisfied_ports(contract) {
        report.push(
            Diagnostic::error("DPCS-DF-006", categories::DATA_FLOW, message)
                .with_object_ref(endpoint)
                .with_remediation("Add a dataFlow entry whose `to` targets this port"),
        );
    }

    for dataset in unreachable_datasets(contract) {
        report.push(
            Diagnostic::error(
                "DPCS-DF-005",
                categories::DATA_FLOW,
                format!("dataset `{dataset}` is unreachable from interface inputs"),
            )
            .with_object_ref(format!("dataFlow.dataset.{dataset}"))
            .with_remediation(
                "Introduce the dataset from an interface input or a sourced upstream step",
            ),
        );
    }

    report
}
