//! Extension validation phase (SPEC Ch 21).

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{
    is_valid_extension_namespace, is_valid_version, ExtensionDefinition, PipelineContract,
};

/// Optional constraints applied when validating extensions.
#[derive(Debug, Clone, Default)]
pub struct ExtensionValidationOptions {
    /// Namespaces that are forbidden by an applicable profile.
    pub forbidden_namespaces: Vec<String>,
}

/// Validate extension field namespaces on a Pipeline Contract.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    validate_with_options(contract, &ExtensionValidationOptions::default())
}

/// Validate extensions with profile-driven constraints.
pub fn validate_with_options(
    contract: &PipelineContract,
    options: &ExtensionValidationOptions,
) -> ValidationReport {
    let mut report = ValidationReport::new();

    for key in contract.extensions.keys() {
        if !is_valid_extension_namespace(key) {
            report.push(
                Diagnostic::error(
                    "DPCS-EXT-001",
                    categories::EXTENSION,
                    format!("extension field `{key}` has an invalid namespace"),
                )
                .with_object_ref(key.as_str())
                .with_remediation("Use an `x-*` prefix, `vendor:name` form, or URI-like namespace"),
            );
            continue;
        }

        if options
            .forbidden_namespaces
            .iter()
            .any(|ns| ns == key || key.starts_with(&format!("{ns}:")))
        {
            report.push(
                Diagnostic::error(
                    "DPCS-EXT-002",
                    categories::EXTENSION,
                    format!("extension namespace `{key}` is forbidden by the active profile"),
                )
                .with_object_ref(key.as_str())
                .with_remediation("Remove the extension or adjust the conformance profile"),
            );
            continue;
        }

        report.push(
            Diagnostic::information(
                "DPCS-EXT-010",
                categories::EXTENSION,
                format!("unrecognized extension `{key}` was preserved"),
            )
            .with_object_ref(key.as_str())
            .with_remediation("Unknown extensions are preserved unless prohibited by a profile"),
        );
    }

    if let Some(policy) = &contract.compatibility {
        if let Some(mode) = &policy.mode {
            let normalized = mode.trim().to_ascii_lowercase();
            if !matches!(
                normalized.as_str(),
                "full" | "fully" | "backward" | "forward" | "conditional" | "none"
            ) {
                report.push(
                    Diagnostic::warning(
                        "DPCS-COMPAT-050",
                        categories::COMPATIBILITY,
                        format!("compatibility mode `{mode}` is not a standard category"),
                    )
                    .with_object_ref("compatibility.mode")
                    .with_remediation("Prefer full, backward, forward, conditional, or none"),
                );
            }
        }
    }

    report
}

/// Validate a standalone [`ExtensionDefinition`] document.
pub fn validate_extension_definition(definition: &ExtensionDefinition) -> ValidationReport {
    let mut report = ValidationReport::new();

    if definition.id.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-EXT-003",
                categories::EXTENSION,
                "extension definition must declare a non-empty id",
            )
            .with_object_ref("id"),
        );
    }
    if !is_valid_extension_namespace(&definition.namespace) {
        report.push(
            Diagnostic::error(
                "DPCS-EXT-004",
                categories::EXTENSION,
                format!("extension namespace `{}` is invalid", definition.namespace),
            )
            .with_object_ref("namespace")
            .with_remediation("Use an `x-*` prefix, `vendor:name` form, or URI-like namespace"),
        );
    }
    if definition.version.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-EXT-005",
                categories::EXTENSION,
                "extension definition must declare a version",
            )
            .with_object_ref("version"),
        );
    } else if !is_valid_version(&definition.version) {
        report.push(
            Diagnostic::error(
                "DPCS-EXT-006",
                categories::EXTENSION,
                format!(
                    "extension version `{}` is not SemVer-compatible",
                    definition.version
                ),
            )
            .with_object_ref("version"),
        );
    }
    if definition.owner.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-EXT-007",
                categories::EXTENSION,
                "extension definition must declare an owner",
            )
            .with_object_ref("owner"),
        );
    }
    if definition.scope.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-EXT-008",
                categories::EXTENSION,
                "extension definition must declare a scope",
            )
            .with_object_ref("scope"),
        );
    }

    report.sort_deterministic();
    report
}
