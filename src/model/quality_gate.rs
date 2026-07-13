//! Quality gate model.

use serde::{Deserialize, Serialize};

use super::ExtensionMap;

/// Quality gate attached to a pipeline, step, or interface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityGate {
    /// Stable quality-gate identifier.
    pub id: String,
    /// Scope to which the gate applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Gate expression or rule identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}
