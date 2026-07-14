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
            if !is_resolvable(contract_ref, &known_refs) {
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

        if let Some(transform_ref) = &step.transform_ref {
            if !is_resolvable(transform_ref, &known_refs) {
                report.push(
                    Diagnostic::error(
                        "DPCS-REF-005",
                        categories::REFERENCE,
                        format!(
                            "step `{}` references unknown transform `{}`",
                            step.id, transform_ref
                        ),
                    )
                    .with_object_ref(format!("steps.{}.transformRef", step.id))
                    .with_remediation(
                        "Declare the transform under contractReferences or use a direct location",
                    ),
                );
            }
        }

        for port in &step.inputs {
            if let Some(contract_ref) = &port.contract_ref {
                if !is_resolvable(contract_ref, &known_refs) {
                    report.push(
                        Diagnostic::error(
                            "DPCS-REF-006",
                            categories::REFERENCE,
                            format!(
                                "step `{}` input port `{}` references unknown contract `{}`",
                                step.id, port.id, contract_ref
                            ),
                        )
                        .with_object_ref(format!("steps.{}.inputs.{}", step.id, port.id))
                        .with_remediation(
                            "Declare the contract under contractReferences or use a direct location",
                        ),
                    );
                }
            }
        }

        for port in &step.outputs {
            if let Some(contract_ref) = &port.contract_ref {
                if !is_resolvable(contract_ref, &known_refs) {
                    report.push(
                        Diagnostic::error(
                            "DPCS-REF-006",
                            categories::REFERENCE,
                            format!(
                                "step `{}` output port `{}` references unknown contract `{}`",
                                step.id, port.id, contract_ref
                            ),
                        )
                        .with_object_ref(format!("steps.{}.outputs.{}", step.id, port.id))
                        .with_remediation(
                            "Declare the contract under contractReferences or use a direct location",
                        ),
                    );
                }
            }
        }
    }

    for (side, ports) in [
        ("inputs", contract.interface.inputs.as_slice()),
        ("outputs", contract.interface.outputs.as_slice()),
    ] {
        for port in ports {
            if let Some(contract_ref) = &port.contract_ref {
                if !is_resolvable(contract_ref, &known_refs) {
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

    for (index, flow) in contract.data_flow.iter().enumerate() {
        if let Some(contract_ref) = &flow.contract_ref {
            if !is_resolvable(contract_ref, &known_refs) {
                report.push(
                    Diagnostic::error(
                        "DPCS-REF-003",
                        categories::REFERENCE,
                        format!("data flow references unknown contract `{contract_ref}`"),
                    )
                    .with_object_ref(format!("dataFlow[{index}].contractRef"))
                    .with_remediation(
                        "Declare the contract under contractReferences or use a direct location",
                    ),
                );
            }
        }
    }

    report
}

fn is_resolvable(value: &str, known_refs: &BTreeSet<&str>) -> bool {
    looks_like_path(value) || known_refs.contains(value)
}

fn looks_like_path(value: &str) -> bool {
    value.contains('/')
}
