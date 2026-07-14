//! Control-flow validation phase.

use std::collections::{BTreeMap, BTreeSet};

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{AnalysisContext, PipelineContract};

#[allow(dead_code)]
/// Validate control-flow dependencies against known steps and other models.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let ctx = AnalysisContext::build(contract);
    validate_with_context(&ctx)
}

/// Validate control flow using a shared analysis context.
pub fn validate_with_context(ctx: &AnalysisContext<'_>) -> ValidationReport {
    let contract = ctx.contract;
    let mut report = ValidationReport::new();
    let step_ids = &ctx.step_ids;

    // Reuse the shared dependency graph's non-control edges via reconstructed
    // deps from graph + data-flow only (control-flow conflict detection).
    let mut deps: BTreeSet<(String, String)> = BTreeSet::new();
    for edge in &contract.graph.edges {
        if edge.from.trim().is_empty() || edge.to.trim().is_empty() {
            continue;
        }
        if step_ids.contains(edge.from.as_str()) && step_ids.contains(edge.to.as_str()) {
            deps.insert((edge.from.clone(), edge.to.clone()));
        }
    }
    for flow in &contract.data_flow {
        if let Some((from_step, to_step)) = ctx.data_flow_step_dependency(&flow.from, &flow.to) {
            deps.insert((from_step, to_step));
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

        // Conflict when a uni-directional graph/data edge points the opposite way.
        // Bidirectional graph cycles are already covered by GRP-004 without CF noise.
        let forward = (flow.to.clone(), flow.from.clone());
        let same = (flow.from.clone(), flow.to.clone());
        if step_ids.contains(flow.from.as_str())
            && step_ids.contains(flow.to.as_str())
            && deps.contains(&forward)
            && !deps.contains(&same)
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
