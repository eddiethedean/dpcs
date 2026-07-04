//! Compatibility policy model.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Declared compatibility policy for a Pipeline Contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityPolicy {
    /// Compatibility mode (for example `backward` or `full`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: IndexMap<String, Value>,
}
