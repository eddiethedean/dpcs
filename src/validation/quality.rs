//! Quality-gate validation phase.

use std::collections::BTreeSet;

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::PipelineContract;

/// Validate quality-gate identifiers and placement.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();
    let mut seen = BTreeSet::new();

    for (index, gate) in contract.quality_gates.iter().enumerate() {
        if gate.id.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-QG-001",
                    categories::STRUCTURAL,
                    "quality gate id must not be empty",
                )
                .with_object_ref(format!("qualityGates[{index}]")),
            );
        } else if !seen.insert(gate.id.as_str()) {
            report.push(
                Diagnostic::error(
                    "DPCS-QG-002",
                    categories::STRUCTURAL,
                    format!("duplicate quality gate id `{}`", gate.id),
                )
                .with_object_ref(format!("qualityGates.{}", gate.id)),
            );
        }
    }

    report
}
