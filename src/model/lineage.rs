//! Pipeline lineage model.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Declared lineage information for a Pipeline Contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PipelineLineage {
    /// Upstream lineage references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upstream: Vec<String>,
    /// Downstream lineage references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub downstream: Vec<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: IndexMap<String, Value>,
}
