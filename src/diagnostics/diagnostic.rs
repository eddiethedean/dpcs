//! Individual diagnostic observations.

use serde::{Deserialize, Serialize};

use super::{DiagnosticStage, Severity};

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
    /// Processing stage that produced the observation.
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
}
