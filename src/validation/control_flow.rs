//! Control-flow validation phase.

use std::collections::{BTreeMap, BTreeSet};

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{data_flow_step_dependency, PipelineContract};

/// Validate control-flow dependencies against known steps and other models.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();
    let step_ids: BTreeSet<&str> = contract.steps.iter().map(|s| s.id.as_str()).collect();

    let mut opposite: BTreeSet<(String, String)> = BTreeSet::new();
    for edge in &contract.graph.edges {
        if edge.from.trim().is_empty() || edge.to.trim().is_empty() {
            continue;
        }
        if step_ids.contains(edge.from.as_str()) && step_ids.contains(edge.to.as_str()) {
            opposite.insert((edge.to.clone(), edge.from.clone()));
        }
    }
    for flow in &contract.data_flow {
        if let Some((from_step, to_step)) =
            data_flow_step_dependency(contract, &flow.from, &flow.to)
        {
            opposite.insert((to_step, from_step));
        }
    }

    let mut seen: BTreeMap<(String, String, Option<String>), usize> = BTreeMap::new();

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
                .with_object_ref(object_ref.clone()),
            );
        }

        let key = (flow.from.clone(), flow.to.clone(), flow.kind.clone());
        if let Some(first_index) = seen.get(&key) {
            report.push(
                Diagnostic::error(
                    "DPCS-CF-005",
                    categories::CONTROL_FLOW,
                    format!(
                        "duplicate control flow from `{}` to `{}`",
                        flow.from, flow.to
                    ),
                )
                .with_object_ref(object_ref.clone())
                .with_remediation(format!(
                    "Remove the duplicate of controlFlow[{first_index}]"
                )),
            );
        } else {
            seen.insert(key, index);
        }

        if step_ids.contains(flow.from.as_str())
            && step_ids.contains(flow.to.as_str())
            && opposite.contains(&(flow.from.clone(), flow.to.clone()))
        {
            report.push(
                Diagnostic::error(
                    "DPCS-CF-004",
                    categories::CONTROL_FLOW,
                    format!(
                        "control flow from `{}` to `{}` conflicts with an opposite graph or data-flow dependency",
                        flow.from, flow.to
                    ),
                )
                .with_object_ref(object_ref)
                .with_remediation(
                    "Align controlFlow with graph.edges and dataFlow dependency directions",
                ),
            );
        }
    }

    report
}
