//! Execution requirements validation phase (SPEC Ch 10).

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::PipelineContract;

/// Validate declared execution requirements for completeness and identifier consistency.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();
    let Some(execution) = &contract.execution else {
        return report;
    };

    for (index, capability) in execution.required_capabilities.iter().enumerate() {
        if capability.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-EXE-001",
                    categories::EXECUTION_REQUIREMENTS,
                    "required capability must not be empty",
                )
                .with_object_ref(format!("execution.requiredCapabilities[{index}]"))
                .with_remediation("Provide a non-empty logical capability identifier"),
            );
        }
    }

    for (index, isolation) in execution.isolation.iter().enumerate() {
        if isolation.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-EXE-002",
                    categories::EXECUTION_REQUIREMENTS,
                    "isolation requirement must not be empty",
                )
                .with_object_ref(format!("execution.isolation[{index}]"))
                .with_remediation(
                    "Use process, container, vm, network, securityDomain, or an extension kind",
                ),
            );
        }
    }

    for (index, dependency) in execution.external_dependencies.iter().enumerate() {
        let object_ref = format!("execution.externalDependencies[{index}]");
        if dependency.id.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-EXE-003",
                    categories::EXECUTION_REQUIREMENTS,
                    "external dependency must declare a non-empty logical service identity",
                )
                .with_object_ref(format!("{object_ref}.id"))
                .with_remediation("Set externalDependencies[].id to a stable service identifier"),
            );
        }
        if dependency.capability.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-EXE-004",
                    categories::EXECUTION_REQUIREMENTS,
                    "external dependency must declare a required capability",
                )
                .with_object_ref(format!("{object_ref}.capability"))
                .with_remediation(
                    "Set externalDependencies[].capability to the required capability",
                ),
            );
        }
    }

    if let Some(resources) = &execution.resources {
        let empty_resource = [
            ("processor", resources.processor.as_deref()),
            ("memory", resources.memory.as_deref()),
            ("storage", resources.storage.as_deref()),
            ("accelerator", resources.accelerator.as_deref()),
            ("bandwidth", resources.bandwidth.as_deref()),
        ]
        .into_iter()
        .any(|(_, value)| value.is_some_and(|v| v.trim().is_empty()));

        if empty_resource {
            report.push(
                Diagnostic::error(
                    "DPCS-EXE-005",
                    categories::EXECUTION_REQUIREMENTS,
                    "resource requirement values must not be empty when declared",
                )
                .with_object_ref("execution.resources")
                .with_remediation(
                    "Omit unused resource fields or provide non-empty logical values",
                ),
            );
        }
    }

    report
}
