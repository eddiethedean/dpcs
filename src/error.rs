//! Error types for parsing and I/O failures.

use std::path::PathBuf;

/// Convenient result alias for fallible DPCS operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that prevent construction of a Canonical Object Model.
///
/// Validation findings are reported through [`crate::ValidationReport`], not
/// this type. `Error` is reserved for parse and I/O failures.
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

    /// YAML document could not be parsed into the Canonical Object Model.
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// JSON document could not be parsed into the Canonical Object Model.
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    /// Document path uses an unsupported extension.
    #[error("unsupported document format for `{path}` (expected .yaml, .yml, or .json)")]
    UnsupportedFormat {
        /// Path with an unsupported extension.
        path: PathBuf,
    },
}
