//! Validation report aggregation and Diagnostic Reports.

use serde::{Deserialize, Serialize};

use super::{Diagnostic, Severity};
use crate::{DPCS_SPEC_VERSION, VERSION};

/// Deterministic collection of diagnostics produced by validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    /// Diagnostics in deterministic order.
    pub diagnostics: Vec<Diagnostic>,
}

impl ValidationReport {
    /// Create an empty report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a diagnostic.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Extend with diagnostics from another report.
    pub fn extend(&mut self, other: ValidationReport) {
        self.diagnostics.extend(other.diagnostics);
    }

    /// Returns `true` when the report contains no error-severity diagnostics.
    pub fn is_valid(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    /// Iterator over error diagnostics.
    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
    }

    /// Iterator over warning diagnostics.
    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
    }

    /// Iterator over informational diagnostics.
    pub fn informations(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Information)
    }

    /// Number of error diagnostics.
    pub fn error_count(&self) -> usize {
        self.errors().count()
    }

    /// Number of warning diagnostics.
    pub fn warning_count(&self) -> usize {
        self.warnings().count()
    }

    /// Number of informational diagnostics.
    pub fn information_count(&self) -> usize {
        self.informations().count()
    }

    /// Sort diagnostics deterministically by id, then object_ref, then message.
    pub fn sort_deterministic(&mut self) {
        self.diagnostics.sort_by(|a, b| {
            a.id.cmp(&b.id)
                .then_with(|| a.object_ref.cmp(&b.object_ref))
                .then_with(|| a.message.cmp(&b.message))
        });
    }

    /// Wrap this report as a [`DiagnosticReport`] with processing metadata.
    pub fn into_diagnostic_report(
        self,
        artifact_id: Option<String>,
        processing_result: ProcessingResult,
    ) -> DiagnosticReport {
        DiagnosticReport {
            processing_result,
            artifact_id,
            implementation: ImplementationMetadata::dpcs_default(),
            diagnostics: self.diagnostics,
        }
    }
}

/// Overall processing result for a diagnostic report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ProcessingResult {
    /// No error-severity diagnostics.
    Valid,
    /// One or more error-severity diagnostics.
    Invalid,
}

impl ProcessingResult {
    /// Derive processing result from a validation report.
    pub fn from_report(report: &ValidationReport) -> Self {
        if report.is_valid() {
            Self::Valid
        } else {
            Self::Invalid
        }
    }
}

impl std::fmt::Display for ProcessingResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Valid => write!(f, "valid"),
            Self::Invalid => write!(f, "invalid"),
        }
    }
}

/// Implementation identity metadata attached to diagnostic reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ImplementationMetadata {
    /// Implementation name.
    pub name: String,
    /// Implementation version.
    pub version: String,
    /// Supported DPCS specification version.
    pub dpcs_spec_version: String,
}

impl ImplementationMetadata {
    /// Metadata for this `dpcs` toolkit.
    pub fn dpcs_default() -> Self {
        Self {
            name: "dpcs".to_owned(),
            version: VERSION.to_owned(),
            dpcs_spec_version: DPCS_SPEC_VERSION.to_owned(),
        }
    }
}

/// Aggregated diagnostic report with processing metadata (SPEC Ch 18 §7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticReport {
    /// Overall processing result.
    pub processing_result: ProcessingResult,
    /// Optional associated artifact identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<String>,
    /// Implementation that produced the report.
    pub implementation: ImplementationMetadata,
    /// Diagnostics in deterministic order.
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticReport {
    /// Build a diagnostic report from a validation report.
    pub fn from_validation(report: ValidationReport, artifact_id: Option<String>) -> Self {
        let processing_result = ProcessingResult::from_report(&report);
        report.into_diagnostic_report(artifact_id, processing_result)
    }

    /// Returns `true` when processing result is valid.
    pub fn is_valid(&self) -> bool {
        self.processing_result == ProcessingResult::Valid
    }
}
