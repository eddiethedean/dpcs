//! Pipeline interface model.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// External boundary of a Pipeline Contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PipelineInterface {
    /// Pipeline inputs.
    #[serde(default)]
    pub inputs: Vec<InterfacePort>,
    /// Pipeline outputs.
    #[serde(default)]
    pub outputs: Vec<InterfacePort>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: IndexMap<String, Value>,
}

/// A single input or output port on the pipeline interface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfacePort {
    /// Stable port identifier.
    pub id: String,
    /// Optional human-readable name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Reference to an external contract (for example ODCS).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_ref: Option<String>,
    /// Logical purpose of the port.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: IndexMap<String, Value>,
}
