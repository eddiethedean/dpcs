//! Control flow model.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Declared control dependency between pipeline elements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlFlow {
    /// Upstream control endpoint.
    pub from: String,
    /// Downstream control endpoint.
    pub to: String,
    /// Optional control-flow kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: IndexMap<String, Value>,
}
