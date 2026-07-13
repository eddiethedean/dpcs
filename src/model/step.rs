//! Pipeline step model.

use serde::{Deserialize, Serialize};

use super::ExtensionMap;

/// Logical unit of work within a Pipeline Contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineStep {
    /// Stable step identifier unique within the contract.
    pub id: String,
    /// Step type (for example `dtcs:transform` or an extension type).
    #[serde(rename = "type")]
    pub step_type: String,
    /// Optional human-readable name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Reference to a contract that defines step behavior.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_ref: Option<String>,
    /// Reference to a DTCS transform contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform_ref: Option<String>,
    /// Declared step inputs.
    #[serde(default)]
    pub inputs: Vec<StepPort>,
    /// Declared step outputs.
    #[serde(default)]
    pub outputs: Vec<StepPort>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Input or output port declared on a pipeline step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepPort {
    /// Port identifier.
    pub id: String,
    /// Optional contract reference for the port.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_ref: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}
