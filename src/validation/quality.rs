//! Quality gate validation phase (SPEC Ch 12).

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::PipelineContract;

/// Validate quality-gate structural constraints beyond identity.
///
/// Identifier presence and uniqueness are owned by the Canonical Object Model
/// phase.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();
    let step_ids = contract.step_ids();
    let reference_ids: std::collections::BTreeSet<&str> = contract
        .contract_references
        .iter()
        .map(|reference| reference.id.as_str())
        .collect();

    for (index, gate) in contract.quality_gates.iter().enumerate() {
        let object_ref = format!("qualityGates[{index}]");

        for legacy_key in ["scope", "rule"] {
            if gate.extensions.contains_key(legacy_key) {
                report.push(
                    Diagnostic::error(
                        "DPCS-QG-009",
                        categories::QUALITY_GATES,
                        format!(
                            "legacy `{legacy_key}` field is not supported; use purpose, criteria, and outcomes"
                        ),
                    )
                    .with_object_ref(format!("{object_ref}.{legacy_key}"))
                    .with_remediation(
                        "Migrate to qualityGates[].purpose, criteria, onSuccess, and onFailure",
                    ),
                );
            }
        }

        if gate.purpose.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-QG-001",
                    categories::QUALITY_GATES,
                    "quality gate must declare a non-empty purpose",
                )
                .with_object_ref(format!("{object_ref}.purpose")),
            );
        }

        if gate.criteria.is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-QG-002",
                    categories::QUALITY_GATES,
                    "quality gate must declare one or more evaluation criteria",
                )
                .with_object_ref(format!("{object_ref}.criteria"))
                .with_remediation("Add at least one criteria entry with expression or contractRef"),
            );
        }

        for (criterion_index, criterion) in gate.criteria.iter().enumerate() {
            let criterion_ref = format!("{object_ref}.criteria[{criterion_index}]");
            let has_expression = criterion
                .expression
                .as_ref()
                .is_some_and(|value| !value.trim().is_empty());
            let has_contract_ref = criterion
                .contract_ref
                .as_ref()
                .is_some_and(|value| !value.trim().is_empty());

            if !has_expression && !has_contract_ref {
                report.push(
                    Diagnostic::error(
                        "DPCS-QG-003",
                        categories::QUALITY_GATES,
                        "quality criterion must declare expression or contractRef",
                    )
                    .with_object_ref(criterion_ref.clone()),
                );
            }

            if let Some(contract_ref) = &criterion.contract_ref {
                if !contract_ref.trim().is_empty() && !reference_ids.contains(contract_ref.as_str())
                {
                    report.push(
                        Diagnostic::error(
                            "DPCS-QG-004",
                            categories::QUALITY_GATES,
                            format!(
                                "quality criterion references unknown contractRef `{contract_ref}`"
                            ),
                        )
                        .with_object_ref(format!("{criterion_ref}.contractRef"))
                        .with_remediation(
                            "Use a contractReferences[].id or remove the contractRef",
                        ),
                    );
                }
            }
        }

        if gate.on_success.as_str().trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-QG-008",
                    categories::QUALITY_GATES,
                    "quality gate onSuccess must not be empty",
                )
                .with_object_ref(format!("{object_ref}.onSuccess")),
            );
        }
        if gate.on_failure.as_str().trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-QG-008",
                    categories::QUALITY_GATES,
                    "quality gate onFailure must not be empty",
                )
                .with_object_ref(format!("{object_ref}.onFailure")),
            );
        }

        if let Some(placement) = &gate.placement {
            if placement.kind.trim().is_empty() {
                report.push(
                    Diagnostic::error(
                        "DPCS-QG-005",
                        categories::QUALITY_GATES,
                        "quality gate placement kind must not be empty",
                    )
                    .with_object_ref(format!("{object_ref}.placement.kind")),
                );
            }

            if placement.targets_step() {
                match &placement.step_id {
                    None => {
                        report.push(
                            Diagnostic::error(
                                "DPCS-QG-006",
                                categories::QUALITY_GATES,
                                "step placement requires stepId",
                            )
                            .with_object_ref(format!("{object_ref}.placement.stepId")),
                        );
                    }
                    Some(step_id) if step_id.trim().is_empty() => {
                        report.push(
                            Diagnostic::error(
                                "DPCS-QG-006",
                                categories::QUALITY_GATES,
                                "step placement requires stepId",
                            )
                            .with_object_ref(format!("{object_ref}.placement.stepId")),
                        );
                    }
                    Some(step_id) if !step_ids.contains(step_id.as_str()) => {
                        report.push(
                            Diagnostic::error(
                                "DPCS-QG-007",
                                categories::QUALITY_GATES,
                                format!(
                                    "quality gate placement references unknown step `{step_id}`"
                                ),
                            )
                            .with_object_ref(format!("{object_ref}.placement.stepId")),
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    report
}
