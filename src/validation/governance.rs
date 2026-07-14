//! Governance metadata validation (SPEC Ch 25).

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{is_known_publication_status, GovernanceMetadata, PipelineContract};

/// Validate governance metadata when present.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();
    if let Some(governance) = &contract.governance {
        report.extend(validate_governance(governance));
    }
    report
}

/// Validate a [`GovernanceMetadata`] block.
pub fn validate_governance(governance: &GovernanceMetadata) -> ValidationReport {
    let mut report = ValidationReport::new();

    if governance
        .owner
        .as_ref()
        .is_some_and(|o| o.trim().is_empty())
    {
        report.push(
            Diagnostic::error(
                "DPCS-GOV-001",
                categories::GOVERNANCE,
                "governance.owner must not be empty when declared",
            )
            .with_object_ref("governance.owner"),
        );
    }

    if governance
        .governing_authority
        .as_ref()
        .is_some_and(|a| a.trim().is_empty())
    {
        report.push(
            Diagnostic::error(
                "DPCS-GOV-002",
                categories::GOVERNANCE,
                "governance.governingAuthority must not be empty when declared",
            )
            .with_object_ref("governance.governingAuthority"),
        );
    }

    if let Some(status) = &governance.publication_status {
        if status.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-GOV-003",
                    categories::GOVERNANCE,
                    "governance.publicationStatus must not be empty when declared",
                )
                .with_object_ref("governance.publicationStatus"),
            );
        } else if !is_known_publication_status(status) {
            report.push(
                Diagnostic::warning(
                    "DPCS-GOV-004",
                    categories::GOVERNANCE,
                    format!("governance.publicationStatus `{status}` is not a recognized status"),
                )
                .with_object_ref("governance.publicationStatus")
                .with_remediation("Prefer draft, published, deprecated, retired, or unpublished"),
            );
        }
    }

    if governance.owner.is_none() && governance.governing_authority.is_none() {
        report.push(
            Diagnostic::warning(
                "DPCS-GOV-005",
                categories::GOVERNANCE,
                "governance block should identify an owner or governingAuthority",
            )
            .with_object_ref("governance")
            .with_remediation("Set governance.owner or governance.governingAuthority"),
        );
    }

    report
}
