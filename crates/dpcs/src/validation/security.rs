//! Security metadata validation (SPEC Ch 24).

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{PipelineContract, SecurityMetadata};

/// Validate security metadata when present.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();
    if let Some(security) = &contract.security {
        report.extend(validate_security(security));
    }
    report
}

/// Validate a [`SecurityMetadata`] block.
pub fn validate_security(security: &SecurityMetadata) -> ValidationReport {
    let mut report = ValidationReport::new();

    for (index, secret) in security.secret_refs.iter().enumerate() {
        let object_ref = format!("security.secretRefs[{index}]");
        if secret.id.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-SEC-001",
                    categories::SECURITY,
                    "secret reference id must not be empty",
                )
                .with_object_ref(format!("{object_ref}.id")),
            );
        }
        if secret.loc.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-SEC-002",
                    categories::SECURITY,
                    "secret reference loc must not be empty",
                )
                .with_object_ref(format!("{object_ref}.loc"))
                .with_remediation("Point loc at an external secret manager reference"),
            );
        }
        // Reject obviously embedded secret material in loc.
        if looks_like_embedded_secret(&secret.loc) {
            report.push(
                Diagnostic::error(
                    "DPCS-SEC-003",
                    categories::SECURITY,
                    "secret reference loc must not embed secret material",
                )
                .with_object_ref(format!("{object_ref}.loc"))
                .with_remediation(
                    "Use an external reference (URI or vault path), not a password or token value",
                ),
            );
        }
        for (key, value) in &secret.extensions {
            if key_suggests_secret(key) {
                if let Some(text) = value.as_str() {
                    if looks_like_embedded_secret(text) {
                        report.push(
                            Diagnostic::error(
                                "DPCS-SEC-003",
                                categories::SECURITY,
                                format!(
                                    "secret reference must not embed secret material in `{key}`"
                                ),
                            )
                            .with_object_ref(format!("{object_ref}.{key}")),
                        );
                    }
                }
            }
        }
    }

    for (index, integrity) in security.integrity_refs.iter().enumerate() {
        let object_ref = format!("security.integrityRefs[{index}]");
        if integrity.kind.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-SEC-004",
                    categories::SECURITY,
                    "integrity reference kind must not be empty",
                )
                .with_object_ref(format!("{object_ref}.kind")),
            );
        }
        if integrity.value.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-SEC-005",
                    categories::SECURITY,
                    "integrity reference value must not be empty",
                )
                .with_object_ref(format!("{object_ref}.value")),
            );
        }
    }

    report
}

fn key_suggests_secret(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("password")
        || key.contains("secret")
        || key.contains("token")
        || key.contains("apikey")
        || key.contains("api_key")
        || key == "key"
}

fn looks_like_embedded_secret(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() {
        return false;
    }
    // External refs typically look like URIs or paths with separators.
    if value.contains("://") || value.starts_with("vault:") || value.starts_with('/') {
        return false;
    }
    // Bare high-entropy-looking tokens without path structure.
    if value.len() >= 16 && !value.contains('/') && !value.contains(':') {
        return true;
    }
    value.to_ascii_lowercase().starts_with("password=")
        || value.to_ascii_lowercase().starts_with("token=")
}
