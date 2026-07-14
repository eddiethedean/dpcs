//! Pipeline lineage model (SPEC Ch 14).

use serde::{Deserialize, Serialize};

use super::ExtensionMap;

/// Declared lineage information for a Pipeline Contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PipelineLineage {
    /// Dataset provenance relationships.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub datasets: Vec<DatasetLineage>,
    /// Step provenance relationships.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<StepLineage>,
    /// Pipeline contract provenance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<PipelineProvenance>,
    /// Optional audit metadata (non-semantic).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit: Option<LineageAudit>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Dataset provenance within a pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DatasetLineage {
    /// Dataset identity.
    pub dataset: String,
    /// Producing pipeline step identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub produced_by: Option<String>,
    /// Consuming pipeline step identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub consumed_by: Vec<String>,
    /// Associated data contract reference id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_ref: Option<String>,
    /// Associated transformation contract reference id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform_ref: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Step provenance within a pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct StepLineage {
    /// Step identity.
    #[serde(rename = "stepId")]
    pub step_id: String,
    /// Predecessor step identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub predecessors: Vec<String>,
    /// Successor step identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub successors: Vec<String>,
    /// Dependency semantics description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependency_kind: Option<String>,
    /// Associated contract reference id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_ref: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Provenance describing contract origin and relationships.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PipelineProvenance {
    /// Originating pipeline contract identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub originating: Option<String>,
    /// Parent pipeline contract identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parents: Vec<String>,
    /// Nested pipeline contract identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nested: Vec<String>,
    /// Imported pipeline contract identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub imported: Vec<String>,
    /// Version history entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub version_history: Vec<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Non-semantic audit metadata for lineage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LineageAudit {
    /// Contract identifiers for audit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contract_ids: Vec<String>,
    /// Version identifiers for audit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub version_ids: Vec<String>,
    /// Timestamps for audit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub timestamps: Vec<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}
