//! Failure semantics model.

use serde::{Deserialize, Serialize};

use super::ExtensionMap;

/// Declared failure behavior for a pipeline scope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailureSemantics {
    /// Stable failure-semantics identifier.
    pub id: String,
    /// Scope to which the semantics apply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Required behavior on failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_failure: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}
