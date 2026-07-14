//! Failure semantics validation phase (SPEC Ch 13).

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::PipelineContract;

/// Validate failure triggers, responses, scope, and retry declarations.
///
/// Identifier presence and uniqueness are owned by the Canonical Object Model
/// phase.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();
    let step_ids = contract.step_ids();

    for (index, semantics) in contract.failure_semantics.iter().enumerate() {
        let object_ref = format!("failureSemantics[{index}]");

        if semantics.scope.kind.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-FS-001",
                    categories::FAILURE_SEMANTICS,
                    "failure semantics scope kind must not be empty",
                )
                .with_object_ref(format!("{object_ref}.scope.kind")),
            );
        }

        if semantics.scope.is_step() {
            match &semantics.scope.step_id {
                None => {
                    report.push(
                        Diagnostic::error(
                            "DPCS-FS-002",
                            categories::FAILURE_SEMANTICS,
                            "step scope requires stepId",
                        )
                        .with_object_ref(format!("{object_ref}.scope.stepId")),
                    );
                }
                Some(step_id) if step_id.trim().is_empty() => {
                    report.push(
                        Diagnostic::error(
                            "DPCS-FS-002",
                            categories::FAILURE_SEMANTICS,
                            "step scope requires stepId",
                        )
                        .with_object_ref(format!("{object_ref}.scope.stepId")),
                    );
                }
                Some(step_id) if !step_ids.contains(step_id.as_str()) => {
                    report.push(
                        Diagnostic::error(
                            "DPCS-FS-003",
                            categories::FAILURE_SEMANTICS,
                            format!("failure semantics scope references unknown step `{step_id}`"),
                        )
                        .with_object_ref(format!("{object_ref}.scope.stepId")),
                    );
                }
                _ => {}
            }
        }

        if semantics.scope.kind == "path" {
            let path_empty = semantics
                .scope
                .path
                .as_ref()
                .map(|value| value.trim().is_empty())
                .unwrap_or(true);
            if path_empty {
                report.push(
                    Diagnostic::error(
                        "DPCS-FS-004",
                        categories::FAILURE_SEMANTICS,
                        "path scope requires a non-empty path",
                    )
                    .with_object_ref(format!("{object_ref}.scope.path")),
                );
            }
        }

        if semantics.triggers.is_empty()
            || semantics
                .triggers
                .iter()
                .all(|trigger| trigger.trim().is_empty())
        {
            report.push(
                Diagnostic::error(
                    "DPCS-FS-005",
                    categories::FAILURE_SEMANTICS,
                    "failure semantics must declare one or more non-empty triggers",
                )
                .with_object_ref(format!("{object_ref}.triggers")),
            );
        } else {
            for (trigger_index, trigger) in semantics.triggers.iter().enumerate() {
                if trigger.trim().is_empty() {
                    report.push(
                        Diagnostic::error(
                            "DPCS-FS-005",
                            categories::FAILURE_SEMANTICS,
                            "failure semantics trigger must not be empty",
                        )
                        .with_object_ref(format!("{object_ref}.triggers[{trigger_index}]")),
                    );
                }
            }
        }

        if semantics.responses.is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-FS-006",
                    categories::FAILURE_SEMANTICS,
                    "failure semantics must declare one or more responses",
                )
                .with_object_ref(format!("{object_ref}.responses")),
            );
        }

        let requires_retry = semantics
            .responses
            .iter()
            .any(crate::model::FailureResponse::is_retry);
        if requires_retry {
            match &semantics.retry {
                None => {
                    report.push(
                        Diagnostic::error(
                            "DPCS-FS-007",
                            categories::FAILURE_SEMANTICS,
                            "retry response requires retry semantics",
                        )
                        .with_object_ref(format!("{object_ref}.retry"))
                        .with_remediation(
                            "Declare failureSemantics[].retry with eligibility or maxAttempts",
                        ),
                    );
                }
                Some(retry)
                    if retry.eligible == Some(false)
                        && retry.max_attempts.is_none()
                        && retry.conditions.is_none() =>
                {
                    report.push(
                        Diagnostic::error(
                            "DPCS-FS-007",
                            categories::FAILURE_SEMANTICS,
                            "retry response requires retry eligibility or retry policy",
                        )
                        .with_object_ref(format!("{object_ref}.retry")),
                    );
                }
                _ => {}
            }
        }
    }

    report
}
