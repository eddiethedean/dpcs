//! Error types for parsing and I/O failures.

use std::path::PathBuf;

use crate::diagnostics::ValidationReport;

/// Convenient result alias for fallible DPCS operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that prevent construction of a Canonical Object Model.
///
/// Validation findings for successfully parsed contracts are reported through
/// [`crate::ValidationReport`]. Invalid documents produce parse-stage diagnostics
/// via [`Error::InvalidDocument`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to read a file from disk.
    #[error("failed to read `{path}`: {source}")]
    Io {
        /// Path that could not be read.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// Document could not be parsed into the Canonical Object Model.
    #[error("{}", format_invalid_document(report))]
    InvalidDocument {
        /// Parse-stage diagnostics for the invalid document.
        report: ValidationReport,
    },

    /// Document path uses an unsupported extension.
    #[error("unsupported document format for `{path}` (expected .yaml, .yml, or .json)")]
    UnsupportedFormat {
        /// Path with an unsupported extension.
        path: PathBuf,
    },

    /// Failed to serialize diagnostics or contract output.
    #[error("{0}")]
    Serialization(String),
}

impl Error {
    /// Returns parse-stage diagnostics when this error represents an invalid document.
    pub fn invalid_document_report(&self) -> Option<&ValidationReport> {
        match self {
            Self::InvalidDocument { report } => Some(report),
            _ => None,
        }
    }
}

fn format_invalid_document(report: &ValidationReport) -> String {
    match report.diagnostics.as_slice() {
        [] => "invalid document".to_owned(),
        [only] => format!("invalid document: {} — {}", only.id, only.message),
        [first, rest @ ..] => format!(
            "invalid document: {} — {} (+{} more)",
            first.id,
            first.message,
            rest.len()
        ),
    }
}
