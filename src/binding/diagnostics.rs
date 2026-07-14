//! Binding-stage diagnostic helpers (SPEC Ch 17 / Ch 18).

use crate::diagnostics::{categories, Diagnostic, ValidationReport};

/// Binding refused because capability matching failed.
pub const BIND_CAPABILITY_FAILED: &str = "DPCS-BIND-001";
/// Unknown or unsupported binding target.
pub const BIND_UNKNOWN_TARGET: &str = "DPCS-BIND-002";
/// Translation could not preserve a mandatory plan semantic.
pub const BIND_TRANSLATION_INCOMPLETE: &str = "DPCS-BIND-003";
/// Failed to write binding artifacts to disk.
pub const BIND_WRITE_FAILED: &str = "DPCS-BIND-004";

/// Error: binding refused after capability evaluation failed.
pub fn capability_gate_failed() -> Diagnostic {
    Diagnostic::binding_error(
        BIND_CAPABILITY_FAILED,
        categories::BINDING,
        "orchestrator binding requires a successful capability match",
    )
    .with_remediation(
        "Resolve missing mandatory capabilities in the profile, or adjust pipeline execution requirements",
    )
}

/// Error: unknown binding target string.
pub fn unknown_target(target: &str) -> Diagnostic {
    Diagnostic::binding_error(
        BIND_UNKNOWN_TARGET,
        categories::BINDING,
        format!("unknown binding target `{target}`"),
    )
    .with_remediation("Use one of: airflow, dagster, prefect, temporal, kubernetes")
}

/// Error: adapter could not represent a mandatory semantic.
pub fn translation_incomplete(message: impl Into<String>) -> Diagnostic {
    Diagnostic::binding_error(BIND_TRANSLATION_INCOMPLETE, categories::BINDING, message)
        .with_remediation("Simplify the plan semantic or choose a different binding target")
}

/// Error: filesystem write failure while emitting artifacts.
pub fn write_failed(message: impl Into<String>) -> Diagnostic {
    Diagnostic::binding_error(BIND_WRITE_FAILED, categories::BINDING, message)
        .with_remediation("Check output path permissions and disk space")
}

/// Wrap capability diagnostics with a binding-stage gate failure.
pub fn report_capability_gate(capability_diagnostics: ValidationReport) -> ValidationReport {
    let mut report = ValidationReport::new();
    report.push(capability_gate_failed());
    report.extend(capability_diagnostics);
    report.sort_deterministic();
    report
}

/// Single-diagnostic binding error report.
pub fn report_error(diagnostic: Diagnostic) -> ValidationReport {
    let mut report = ValidationReport::new();
    report.push(diagnostic);
    report.sort_deterministic();
    report
}
