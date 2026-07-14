//! Pipeline graph model.

use serde::{Deserialize, Serialize};

use super::{ExtensionMap, Metadata};

/// Directed graph describing step relationships.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PipelineGraph {
    /// Declared pipeline entry step identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entry_points: Vec<String>,
    /// Declared pipeline exit step identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exit_points: Vec<String>,
    /// Optional graph metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// Graph edges between steps or control points.
    #[serde(default)]
    pub edges: Vec<GraphEdge>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// A single directed edge in the pipeline graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge {
    /// Source node identifier.
    pub from: String,
    /// Destination node identifier.
    pub to: String,
    /// Optional edge kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}
