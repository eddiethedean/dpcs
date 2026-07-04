//! Failure-semantics validation phase.

use std::collections::BTreeSet;

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::PipelineContract;

/// Validate failure-semantics identifiers and scopes.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();
    let mut seen = BTreeSet::new();

    for (index, semantics) in contract.failure_semantics.iter().enumerate() {
        if semantics.id.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-FS-001",
                    categories::STRUCTURAL,
                    "failure semantics id must not be empty",
                )
                .with_object_ref(format!("failureSemantics[{index}]")),
            );
        } else if !seen.insert(semantics.id.as_str()) {
            report.push(
                Diagnostic::error(
                    "DPCS-FS-002",
                    categories::STRUCTURAL,
                    format!("duplicate failure semantics id `{}`", semantics.id),
                )
                .with_object_ref(format!("failureSemantics.{}", semantics.id)),
            );
        }
    }

    report
}
