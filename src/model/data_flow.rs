//! Data flow model.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Declared movement of data between addressable endpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataFlow {
    /// Source endpoint reference.
    pub from: String,
    /// Destination endpoint reference.
    pub to: String,
    /// Optional dataset identifier carried by the flow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataset: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: IndexMap<String, Value>,
}
