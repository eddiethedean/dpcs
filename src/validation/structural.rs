//! Structural validation phase.

use std::collections::BTreeSet;

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::PipelineContract;

/// Validate structural constraints such as unique identifiers.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();

    let mut seen_steps = BTreeSet::new();
    for (index, step) in contract.steps.iter().enumerate() {
        if step.id.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-STR-001",
                    categories::STRUCTURAL,
                    "pipeline step id must not be empty",
                )
                .with_object_ref(format!("steps[{index}]"))
                .with_remediation("Provide a unique non-empty step identifier"),
            );
            continue;
        }

        if !seen_steps.insert(step.id.as_str()) {
            report.push(
                Diagnostic::error(
                    "DPCS-STR-002",
                    categories::STRUCTURAL,
                    format!("duplicate pipeline step id `{}`", step.id),
                )
                .with_object_ref(format!("steps.{}", step.id))
                .with_remediation("Ensure every pipeline step has a unique identifier"),
            );
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

    let mut seen_inputs = BTreeSet::new();
    for (index, port) in contract.interface.inputs.iter().enumerate() {
        if port.id.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-STR-004",
                    categories::STRUCTURAL,
                    "interface input id must not be empty",
                )
                .with_object_ref(format!("interface.inputs[{index}]")),
            );
        } else if !seen_inputs.insert(port.id.as_str()) {
            report.push(
                Diagnostic::error(
                    "DPCS-STR-005",
                    categories::STRUCTURAL,
                    format!("duplicate interface input id `{}`", port.id),
                )
                .with_object_ref(format!("interface.inputs.{}", port.id)),
            );
        }
    }

    let mut seen_outputs = BTreeSet::new();
    for (index, port) in contract.interface.outputs.iter().enumerate() {
        if port.id.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-STR-006",
                    categories::STRUCTURAL,
                    "interface output id must not be empty",
                )
                .with_object_ref(format!("interface.outputs[{index}]")),
            );
        } else if !seen_outputs.insert(port.id.as_str()) {
            report.push(
                Diagnostic::error(
                    "DPCS-STR-007",
                    categories::STRUCTURAL,
                    format!("duplicate interface output id `{}`", port.id),
                )
                .with_object_ref(format!("interface.outputs.{}", port.id)),
            );
        }
    }

    let mut seen_refs = BTreeSet::new();
    for (index, reference) in contract.contract_references.iter().enumerate() {
        if reference.id.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-STR-008",
                    categories::STRUCTURAL,
                    "contract reference id must not be empty",
                )
                .with_object_ref(format!("contractReferences[{index}]")),
            );
        } else if !seen_refs.insert(reference.id.as_str()) {
            report.push(
                Diagnostic::error(
                    "DPCS-STR-009",
                    categories::STRUCTURAL,
                    format!("duplicate contract reference id `{}`", reference.id),
                )
                .with_object_ref(format!("contractReferences.{}", reference.id)),
            );
        }
    }

    report
}
