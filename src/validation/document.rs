//! Document validation phase.

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{is_present_version, PipelineContract};
use crate::DPCS_SPEC_VERSION;

/// Validate document-level required fields and version declarations.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();

    if contract.dpcs_version.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-DOC-001",
                categories::DOCUMENT,
                "dpcsVersion must not be empty",
            )
            .with_object_ref("dpcsVersion")
            .with_remediation("Set dpcsVersion to a supported DPCS specification version"),
        );
    } else if contract.dpcs_version != DPCS_SPEC_VERSION && !contract.dpcs_version.starts_with("1.")
    {
        report.push(
            Diagnostic::warning(
                "DPCS-DOC-002",
                categories::DOCUMENT,
                format!(
                    "dpcsVersion `{}` may not be fully supported by this implementation",
                    contract.dpcs_version
                ),
            )
            .with_object_ref("dpcsVersion")
            .with_remediation(format!("Use dpcsVersion `{DPCS_SPEC_VERSION}`")),
        );
    }

    if contract.id.trim().is_empty() {
        report.push(
            Diagnostic::error("DPCS-DOC-003", categories::DOCUMENT, "id must not be empty")
                .with_object_ref("id")
                .with_remediation("Provide a stable pipeline identity"),
        );
    }

    if !is_present_version(&contract.version) {
        report.push(
            Diagnostic::error(
                "DPCS-DOC-004",
                categories::DOCUMENT,
                "version must not be empty",
            )
            .with_object_ref("version")
            .with_remediation("Provide a pipeline contract version"),
        );
    }

    report
}
