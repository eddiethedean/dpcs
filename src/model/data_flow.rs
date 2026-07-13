//! Data flow model.

use serde::{Deserialize, Serialize};

use super::ExtensionMap;

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
    /// Optional associated contract reference identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_ref: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}
