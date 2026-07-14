//! Failure semantics validation phase (SPEC Ch 13).

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{FailureResponse, PipelineContract, RetrySemantics};

/// Validate failure triggers, responses, scope, and retry declarations.
///
/// Identifier presence and uniqueness are owned by the Canonical Object Model
/// phase.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();
    let step_ids = contract.step_ids();

    for (index, semantics) in contract.failure_semantics.iter().enumerate() {
        let object_ref = format!("failureSemantics[{index}]");

        if semantics.extensions.contains_key("onFailure") {
            report.push(
                Diagnostic::error(
                    "DPCS-FS-008",
                    categories::FAILURE_SEMANTICS,
                    "legacy onFailure field is not supported; use responses",
                )
                .with_object_ref(format!("{object_ref}.onFailure"))
                .with_remediation(
                    "Replace onFailure with failureSemantics[].responses and optional retry",
                ),
            );
        }

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

        if semantics.scope.kind.eq_ignore_ascii_case("path") {
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

        if semantics.responses.is_empty()
            || semantics
                .responses
                .iter()
                .all(|response| response.as_str().trim().is_empty())
        {
            report.push(
                Diagnostic::error(
                    "DPCS-FS-006",
                    categories::FAILURE_SEMANTICS,
                    "failure semantics must declare one or more non-empty responses",
                )
                .with_object_ref(format!("{object_ref}.responses")),
            );
        } else {
            for (response_index, response) in semantics.responses.iter().enumerate() {
                if response.as_str().trim().is_empty() {
                    report.push(
                        Diagnostic::error(
                            "DPCS-FS-006",
                            categories::FAILURE_SEMANTICS,
                            "failure semantics response must not be empty",
                        )
                        .with_object_ref(format!("{object_ref}.responses[{response_index}]")),
                    );
                }
            }
        }

        let requires_retry = semantics.responses.iter().any(FailureResponse::is_retry);
        if requires_retry && !retry_is_meaningful(semantics.retry.as_ref()) {
            report.push(
                Diagnostic::error(
                    "DPCS-FS-007",
                    categories::FAILURE_SEMANTICS,
                    "retry response requires meaningful retry semantics",
                )
                .with_object_ref(format!("{object_ref}.retry"))
                .with_remediation(
                    "Declare retry.eligible=true and/or maxAttempts, conditions, delayPolicy, or termination",
                ),
            );
        }
    }

    report
}

fn retry_is_meaningful(retry: Option<&RetrySemantics>) -> bool {
    let Some(retry) = retry else {
        return false;
    };
    if retry.eligible == Some(false) {
        return false;
    }
    retry.eligible == Some(true)
        || retry.max_attempts.is_some_and(|attempts| attempts > 0)
        || retry
            .conditions
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
        || retry
            .delay_policy
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
        || retry
            .termination
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
}
