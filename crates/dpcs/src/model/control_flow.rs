//! Control flow model.

use serde::{Deserialize, Serialize};

use super::ExtensionMap;

/// Declared control dependency between pipeline elements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
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
    pub extensions: ExtensionMap,
}
