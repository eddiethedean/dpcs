//! Structural validation phase.

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::PipelineContract;

/// Validate non-identity structural constraints.
///
/// Identifier presence and uniqueness are owned by the Canonical Object Model
/// phase. This phase covers remaining structural shape such as step types.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();

    for step in &contract.steps {
        if step.id.trim().is_empty() {
            continue;
        }

        if step.step_type.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-STR-003",
                    categories::STRUCTURAL,
                    format!("pipeline step `{}` has an empty type", step.id),
                )
                .with_object_ref(format!("steps.{}.type", step.id))
                .with_remediation("Provide a step type"),
            );
        }
    }

    report
}
