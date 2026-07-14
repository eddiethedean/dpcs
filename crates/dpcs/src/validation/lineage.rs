//! Pipeline lineage validation phase (SPEC Ch 14).

use std::collections::BTreeSet;

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{AnalysisContext, PipelineContract};

#[allow(dead_code)]
/// Validate lineage dataset, step, and contract references.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let ctx = AnalysisContext::build(contract);
    validate_with_context(&ctx)
}

/// Validate lineage using a shared analysis context.
pub fn validate_with_context(ctx: &AnalysisContext<'_>) -> ValidationReport {
    let contract = ctx.contract;
    let mut report = ValidationReport::new();
    let Some(lineage) = &contract.lineage else {
        return report;
    };

    for legacy_key in ["upstream", "downstream"] {
        if lineage.extensions.contains_key(legacy_key) {
            report.push(
                Diagnostic::error(
                    "DPCS-LIN-016",
                    categories::LINEAGE,
                    format!(
                        "legacy `{legacy_key}` lineage field is not supported; use datasets and steps"
                    ),
                )
                .with_object_ref(format!("lineage.{legacy_key}"))
                .with_remediation(
                    "Migrate upstream/downstream stubs to lineage.datasets and lineage.steps",
                ),
            );
        }
    }

    let step_ids = &ctx.step_ids;
    let reference_ids: BTreeSet<&str> = contract
        .contract_references
        .iter()
        .map(|reference| reference.id.as_str())
        .collect();
    let known_datasets: BTreeSet<&str> = contract
        .data_flow
        .iter()
        .filter_map(|flow| flow.dataset.as_deref())
        .filter(|dataset| !dataset.trim().is_empty())
        .collect();

    for (index, dataset) in lineage.datasets.iter().enumerate() {
        let object_ref = format!("lineage.datasets[{index}]");
        if dataset.dataset.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-LIN-001",
                    categories::LINEAGE,
                    "dataset lineage must declare a non-empty dataset identity",
                )
                .with_object_ref(format!("{object_ref}.dataset")),
            );
        } else if !known_datasets.contains(dataset.dataset.as_str()) {
            report.push(
                Diagnostic::warning(
                    "DPCS-LIN-002",
                    categories::LINEAGE,
                    format!(
                        "dataset lineage references dataset `{}` not present in dataFlow",
                        dataset.dataset
                    ),
                )
                .with_object_ref(format!("{object_ref}.dataset")),
            );
        }

        if let Some(produced_by) = &dataset.produced_by {
            if produced_by.trim().is_empty() {
                report.push(
                    Diagnostic::error(
                        "DPCS-LIN-003",
                        categories::LINEAGE,
                        "dataset lineage producedBy must not be empty when declared",
                    )
                    .with_object_ref(format!("{object_ref}.producedBy")),
                );
            } else if !step_ids.contains(produced_by.as_str()) {
                report.push(
                    Diagnostic::error(
                        "DPCS-LIN-004",
                        categories::LINEAGE,
                        format!(
                            "dataset lineage producedBy references unknown step `{produced_by}`"
                        ),
                    )
                    .with_object_ref(format!("{object_ref}.producedBy")),
                );
            }
        }

        for (consumer_index, consumer) in dataset.consumed_by.iter().enumerate() {
            if consumer.trim().is_empty() {
                report.push(
                    Diagnostic::error(
                        "DPCS-LIN-005",
                        categories::LINEAGE,
                        "dataset lineage consumedBy entry must not be empty",
                    )
                    .with_object_ref(format!("{object_ref}.consumedBy[{consumer_index}]")),
                );
            } else if !step_ids.contains(consumer.as_str()) {
                report.push(
                    Diagnostic::error(
                        "DPCS-LIN-006",
                        categories::LINEAGE,
                        format!("dataset lineage consumedBy references unknown step `{consumer}`"),
                    )
                    .with_object_ref(format!("{object_ref}.consumedBy[{consumer_index}]")),
                );
            }
        }

        push_unknown_ref(
            &mut report,
            &reference_ids,
            dataset.contract_ref.as_deref(),
            format!("{object_ref}.contractRef"),
            "DPCS-LIN-007",
            "dataset lineage contractRef",
        );
        push_unknown_ref(
            &mut report,
            &reference_ids,
            dataset.transform_ref.as_deref(),
            format!("{object_ref}.transformRef"),
            "DPCS-LIN-008",
            "dataset lineage transformRef",
        );
    }

    for (index, step) in lineage.steps.iter().enumerate() {
        let object_ref = format!("lineage.steps[{index}]");
        if step.step_id.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-LIN-009",
                    categories::LINEAGE,
                    "step lineage must declare a non-empty stepId",
                )
                .with_object_ref(format!("{object_ref}.stepId")),
            );
        } else if !step_ids.contains(step.step_id.as_str()) {
            report.push(
                Diagnostic::error(
                    "DPCS-LIN-010",
                    categories::LINEAGE,
                    format!("step lineage references unknown step `{}`", step.step_id),
                )
                .with_object_ref(format!("{object_ref}.stepId")),
            );
        }

        for (pred_index, predecessor) in step.predecessors.iter().enumerate() {
            if predecessor.trim().is_empty() {
                report.push(
                    Diagnostic::error(
                        "DPCS-LIN-011",
                        categories::LINEAGE,
                        "step lineage predecessor must not be empty",
                    )
                    .with_object_ref(format!("{object_ref}.predecessors[{pred_index}]")),
                );
            } else if !step_ids.contains(predecessor.as_str()) {
                report.push(
                    Diagnostic::error(
                        "DPCS-LIN-012",
                        categories::LINEAGE,
                        format!("step lineage predecessor references unknown step `{predecessor}`"),
                    )
                    .with_object_ref(format!("{object_ref}.predecessors[{pred_index}]")),
                );
            }
        }

        for (succ_index, successor) in step.successors.iter().enumerate() {
            if successor.trim().is_empty() {
                report.push(
                    Diagnostic::error(
                        "DPCS-LIN-013",
                        categories::LINEAGE,
                        "step lineage successor must not be empty",
                    )
                    .with_object_ref(format!("{object_ref}.successors[{succ_index}]")),
                );
            } else if !step_ids.contains(successor.as_str()) {
                report.push(
                    Diagnostic::error(
                        "DPCS-LIN-014",
                        categories::LINEAGE,
                        format!("step lineage successor references unknown step `{successor}`"),
                    )
                    .with_object_ref(format!("{object_ref}.successors[{succ_index}]")),
                );
            }
        }

        push_unknown_ref(
            &mut report,
            &reference_ids,
            step.contract_ref.as_deref(),
            format!("{object_ref}.contractRef"),
            "DPCS-LIN-015",
            "step lineage contractRef",
        );
    }

    report
}

fn push_unknown_ref(
    report: &mut ValidationReport,
    reference_ids: &BTreeSet<&str>,
    value: Option<&str>,
    object_ref: String,
    diagnostic_id: &str,
    context: &str,
) {
    let Some(reference) = value else {
        return;
    };
    if reference.trim().is_empty() {
        report.push(
            Diagnostic::error(
                diagnostic_id,
                categories::LINEAGE,
                format!("{context} must not be empty when declared"),
            )
            .with_object_ref(object_ref),
        );
        return;
    }
    if !reference_ids.contains(reference) {
        report.push(
            Diagnostic::error(
                diagnostic_id,
                categories::LINEAGE,
                format!("{context} references unknown id `{reference}`"),
            )
            .with_object_ref(object_ref)
            .with_remediation("Use a contractReferences[].id or remove the reference"),
        );
    }
}
