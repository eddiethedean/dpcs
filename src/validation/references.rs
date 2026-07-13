//! Contract reference validation phase.

use std::collections::BTreeSet;

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::PipelineContract;

/// Validate that contract references are well-formed and resolvable by id.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();
    let known_refs: BTreeSet<&str> = contract
        .contract_references
        .iter()
        .map(|r| r.id.as_str())
        .collect();

    for (index, reference) in contract.contract_references.iter().enumerate() {
        if reference.reference_type.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-REF-001",
                    categories::REFERENCE,
                    format!("contract reference `{}` has an empty type", reference.id),
                )
                .with_object_ref(format!("contractReferences[{index}].type")),
            );
        }
        if reference.location.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-REF-002",
                    categories::REFERENCE,
                    format!(
                        "contract reference `{}` has an empty location",
                        reference.id
                    ),
                )
                .with_object_ref(format!("contractReferences[{index}].location")),
            );
        }
    }

    for step in &contract.steps {
        if let Some(contract_ref) = &step.contract_ref {
            if !looks_like_path(contract_ref) && !known_refs.contains(contract_ref.as_str()) {
                report.push(
                    Diagnostic::error(
                        "DPCS-REF-003",
                        categories::REFERENCE,
                        format!(
                            "step `{}` references unknown contract `{}`",
                            step.id, contract_ref
                        ),
                    )
                    .with_object_ref(format!("steps.{}.contractRef", step.id))
                    .with_remediation(
                        "Declare the contract under contractReferences or use a direct location",
                    ),
                );
            }
        }
    }

    for (side, ports) in [
        ("inputs", contract.interface.inputs.as_slice()),
        ("outputs", contract.interface.outputs.as_slice()),
    ] {
        for port in ports {
            if let Some(contract_ref) = &port.contract_ref {
                if !looks_like_path(contract_ref) && !known_refs.contains(contract_ref.as_str()) {
                    report.push(
                        Diagnostic::error(
                            "DPCS-REF-004",
                            categories::REFERENCE,
                            format!(
                                "interface port `{}` references unknown contract `{}`",
                                port.id, contract_ref
                            ),
                        )
                        .with_object_ref(format!("interface.{side}.{}", port.id)),
                    );
                }
            }
        }
    }

    report
}

fn looks_like_path(value: &str) -> bool {
    value.contains('/')
        || value.ends_with(".yaml")
        || value.ends_with(".yml")
        || value.ends_with(".json")
}
