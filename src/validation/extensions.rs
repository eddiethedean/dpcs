//! Extension validation phase.

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{is_reserved_root_field, PipelineContract};

/// Validate extension field namespaces.
///
/// Extension keys that collide with reserved root fields are rejected. Other
/// extension fields are preserved and accepted in this release.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();

    for key in contract.extensions.keys() {
        if is_reserved_root_field(key) {
            report.push(
                Diagnostic::error(
                    "DPCS-EXT-001",
                    categories::EXTENSION,
                    format!("extension key `{key}` collides with a reserved root field"),
                )
                .with_object_ref(key.clone())
                .with_remediation("Use a namespaced extension key such as `x-vendor.field`"),
            );
        }
    }

    report
}
