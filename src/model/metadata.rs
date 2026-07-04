//! Pipeline metadata.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Descriptive metadata attached to a Pipeline Contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// Free-form description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owning team or organization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Classification tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Extension metadata.
    #[serde(default, flatten)]
    pub extensions: IndexMap<String, Value>,
}
