//! Document validation phase.

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::PipelineContract;
use crate::DPCS_SPEC_VERSION;

/// Validate document-level version support warnings.
///
/// Required identity presence is owned by the Canonical Object Model phase.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();

    if !contract.dpcs_version.trim().is_empty()
        && contract.dpcs_version != DPCS_SPEC_VERSION
        && !contract.dpcs_version.starts_with("1.")
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

    report
}
