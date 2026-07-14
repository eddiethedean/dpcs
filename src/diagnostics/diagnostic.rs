//! Individual diagnostic observations.

use serde::{Deserialize, Serialize};

use super::{DiagnosticStage, Severity};
use crate::diagnostics::ValidationReport;

/// A single deterministic diagnostic observation.
///
/// Diagnostics describe observations only and must not change pipeline
/// semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    /// Stable diagnostic identifier (for example `DPCS-VAL-001`).
    pub id: String,
    /// Severity of the observation.
    pub severity: Severity,
    /// Processing stage that produced the diagnostic.
    pub stage: DiagnosticStage,
    /// Category of the observation.
    pub category: String,
    /// Human-readable message.
    pub message: String,
    /// Optional object reference within the contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_ref: Option<String>,
    /// Optional remediation guidance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
    /// Optional source location in the original document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_location: Option<String>,
    /// Related diagnostic identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_diagnostics: Vec<String>,
    /// Implementation-specific metadata (string map).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::BTreeMap<String, String>>,
}

impl Diagnostic {
    /// Create an error diagnostic.
    pub fn error(
        id: impl Into<String>,
        category: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            severity: Severity::Error,
            stage: DiagnosticStage::Validation,
            category: category.into(),
            message: message.into(),
            object_ref: None,
            remediation: None,
            source_location: None,
            related_diagnostics: Vec::new(),
            metadata: None,
        }
    }

    /// Create a parse-stage error diagnostic.
    pub fn parse_error(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            severity: Severity::Error,
            stage: DiagnosticStage::Parse,
            category: crate::diagnostics::categories::SYNTAX.to_owned(),
            message: message.into(),
            object_ref: None,
            remediation: None,
            source_location: None,
            related_diagnostics: Vec::new(),
            metadata: None,
        }
    }

    /// Create a warning diagnostic.
    pub fn warning(
        id: impl Into<String>,
        category: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            severity: Severity::Warning,
            stage: DiagnosticStage::Validation,
            category: category.into(),
            message: message.into(),
            object_ref: None,
            remediation: None,
            source_location: None,
            related_diagnostics: Vec::new(),
            metadata: None,
        }
    }

    /// Create an informational diagnostic.
    pub fn information(
        id: impl Into<String>,
        category: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            severity: Severity::Information,
            stage: DiagnosticStage::Validation,
            category: category.into(),
            message: message.into(),
            object_ref: None,
            remediation: None,
            source_location: None,
            related_diagnostics: Vec::new(),
            metadata: None,
        }
    }

    /// Create a planning-stage error diagnostic.
    pub fn planning_error(
        id: impl Into<String>,
        category: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            severity: Severity::Error,
            stage: DiagnosticStage::Planning,
            category: category.into(),
            message: message.into(),
            object_ref: None,
            remediation: None,
            source_location: None,
            related_diagnostics: Vec::new(),
            metadata: None,
        }
    }

    /// Create a capability-evaluation-stage error diagnostic.
    pub fn capability_error(
        id: impl Into<String>,
        category: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            severity: Severity::Error,
            stage: DiagnosticStage::CapabilityEvaluation,
            category: category.into(),
            message: message.into(),
            object_ref: None,
            remediation: None,
            source_location: None,
            related_diagnostics: Vec::new(),
            metadata: None,
        }
    }

    /// Create a capability-evaluation-stage warning diagnostic.
    pub fn capability_warning(
        id: impl Into<String>,
        category: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            severity: Severity::Warning,
            stage: DiagnosticStage::CapabilityEvaluation,
            category: category.into(),
            message: message.into(),
            object_ref: None,
            remediation: None,
            source_location: None,
            related_diagnostics: Vec::new(),
            metadata: None,
        }
    }

    /// Create an orchestrator-binding-stage error diagnostic.
    pub fn binding_error(
        id: impl Into<String>,
        category: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            severity: Severity::Error,
            stage: DiagnosticStage::OrchestratorBinding,
            category: category.into(),
            message: message.into(),
            object_ref: None,
            remediation: None,
            source_location: None,
            related_diagnostics: Vec::new(),
            metadata: None,
        }
    }

    /// Create a compatibility-analysis-stage error diagnostic.
    pub fn compatibility_error(
        id: impl Into<String>,
        category: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            severity: Severity::Error,
            stage: DiagnosticStage::CompatibilityAnalysis,
            category: category.into(),
            message: message.into(),
            object_ref: None,
            remediation: None,
            source_location: None,
            related_diagnostics: Vec::new(),
            metadata: None,
        }
    }

    /// Create a compatibility-analysis-stage warning diagnostic.
    pub fn compatibility_warning(
        id: impl Into<String>,
        category: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            severity: Severity::Warning,
            stage: DiagnosticStage::CompatibilityAnalysis,
            category: category.into(),
            message: message.into(),
            object_ref: None,
            remediation: None,
            source_location: None,
            related_diagnostics: Vec::new(),
            metadata: None,
        }
    }

    /// Attach an object reference.
    pub fn with_object_ref(mut self, object_ref: impl Into<String>) -> Self {
        self.object_ref = Some(object_ref.into());
        self
    }

    /// Attach remediation guidance.
    pub fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = Some(remediation.into());
        self
    }

    /// Attach a source location in the original document.
    pub fn with_source_location(mut self, source_location: impl Into<String>) -> Self {
        self.source_location = Some(source_location.into());
        self
    }

    /// Attach related diagnostic identifiers.
    pub fn with_related(mut self, related: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.related_diagnostics = related.into_iter().map(Into::into).collect();
        self
    }
}

/// Validate the structural shape of a diagnostic observation.
///
/// Returns findings when required fields are missing or identifiers look
/// malformed. Does not alter the diagnostic under inspection.
pub fn validate_diagnostic(diagnostic: &Diagnostic) -> ValidationReport {
    let mut report = ValidationReport::new();

    if diagnostic.id.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-DIAG-001",
                crate::diagnostics::categories::DOCUMENT,
                "diagnostic identifier must not be empty",
            )
            .with_remediation("Provide a stable diagnostic id such as `DPCS-…`"),
        );
    } else if !diagnostic.id.starts_with("DPCS-") {
        report.push(
            Diagnostic::warning(
                "DPCS-DIAG-002",
                crate::diagnostics::categories::DOCUMENT,
                format!(
                    "diagnostic identifier `{}` does not use the DPCS- prefix",
                    diagnostic.id
                ),
            )
            .with_object_ref(&diagnostic.id)
            .with_remediation("Prefer identifiers of the form `DPCS-<AREA>-<NNN>`"),
        );
    }

    if diagnostic.category.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-DIAG-003",
                crate::diagnostics::categories::DOCUMENT,
                "diagnostic category must not be empty",
            )
            .with_object_ref(&diagnostic.id),
        );
    }

    if diagnostic.message.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-DIAG-004",
                crate::diagnostics::categories::DOCUMENT,
                "diagnostic message must not be empty",
            )
            .with_object_ref(&diagnostic.id),
        );
    }

    report
}
