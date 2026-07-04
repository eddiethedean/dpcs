//! Validation report aggregation.

use serde::{Deserialize, Serialize};

use super::{Diagnostic, Severity};

/// Deterministic collection of diagnostics produced by validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
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

    /// Number of error diagnostics.
    pub fn error_count(&self) -> usize {
        self.errors().count()
    }

    /// Number of warning diagnostics.
    pub fn warning_count(&self) -> usize {
        self.warnings().count()
    }

    /// Sort diagnostics deterministically by id, then object_ref, then message.
    pub fn sort_deterministic(&mut self) {
        self.diagnostics.sort_by(|a, b| {
            a.id.cmp(&b.id)
                .then_with(|| a.object_ref.cmp(&b.object_ref))
                .then_with(|| a.message.cmp(&b.message))
        });
    }
}
