//! Contract reference model.

use serde::{Deserialize, Serialize};

use super::ExtensionMap;

/// Reference to an external contract artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractReference {
    /// Stable reference identifier.
    pub id: String,
    /// Contract type (for example `odcs` or `dtcs`).
    #[serde(rename = "type")]
    pub reference_type: String,
    /// Location of the referenced contract.
    pub location: String,
    /// Optional version constraint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}
