//! Data-flow validation phase.

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{
    data_flow_endpoint_known, unreachable_datasets, unsatisfied_ports, DependencyGraph,
    PipelineContract,
};

/// Validate data-flow endpoints, dataset identity, wiring, and reachability.
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
                .with_object_ref(object_ref.clone()),
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

    let has_cycle = DependencyGraph::from_contract(contract).has_cycle();
    if !has_cycle {
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
    }

    report
}
