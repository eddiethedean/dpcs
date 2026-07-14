//! Diagnostic severity levels.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Severity of a diagnostic observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    /// Informational observation that does not affect validity.
    Information,
    /// Non-fatal issue that should be reviewed.
    Warning,
    /// Error that makes the contract invalid.
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Information => write!(f, "information"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
        }
    }
}
