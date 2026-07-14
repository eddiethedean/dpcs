//! Root Pipeline Contract object.

use serde::{Deserialize, Serialize};

use super::ExtensionMap;

use super::{
    CompatibilityPolicy, ContractReference, ControlFlow, DataFlow, ExecutionRequirements,
    FailureSemantics, IdentityCatalog, Metadata, PipelineGraph, PipelineIdentity,
    PipelineInterface, PipelineLineage, PipelineStep, QualityGate, SchedulingIntent,
};
use crate::diagnostics::ValidationReport;
use crate::error::Result;
use crate::validation;

/// Root Canonical Object Model for a DPCS Pipeline Contract.
///
/// Field names follow the camelCase serialization used in `SPEC.md` examples.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineContract {
    /// DPCS specification version this contract targets.
    pub dpcs_version: String,
    /// Stable pipeline identity.
    pub id: String,
    /// Human-readable pipeline name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Pipeline contract version.
    pub version: String,
    /// Optional descriptive metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// External pipeline interface.
    pub interface: PipelineInterface,
    /// Pipeline graph topology.
    pub graph: PipelineGraph,
    /// Ordered pipeline steps.
    #[serde(default)]
    pub steps: Vec<PipelineStep>,
    /// External contract references (ODCS, DTCS, and others).
    #[serde(default)]
    pub contract_references: Vec<ContractReference>,
    /// Declared data-flow relationships.
    #[serde(default)]
    pub data_flow: Vec<DataFlow>,
    /// Declared control-flow relationships.
    #[serde(default)]
    pub control_flow: Vec<ControlFlow>,
    /// Optional execution requirements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionRequirements>,
    /// Declared scheduling intents.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scheduling: Vec<SchedulingIntent>,
    /// Quality gates attached to the pipeline.
    #[serde(default)]
    pub quality_gates: Vec<QualityGate>,
    /// Failure semantics attached to the pipeline.
    #[serde(default)]
    pub failure_semantics: Vec<FailureSemantics>,
    /// Optional lineage declarations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage: Option<PipelineLineage>,
    /// Optional compatibility policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<CompatibilityPolicy>,
    /// Extension fields preserved from the source document.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

impl PipelineContract {
    /// Parse a Pipeline Contract from a YAML file.
    pub fn from_yaml_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        crate::parser::parse_yaml_file(path)
    }

    /// Parse a Pipeline Contract from a JSON file.
    pub fn from_json_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        crate::parser::parse_json_file(path)
    }

    /// Parse a Pipeline Contract from a YAML string.
    pub fn from_yaml_str(input: &str) -> Result<Self> {
        crate::parser::parse_yaml(input)
    }

    /// Parse a Pipeline Contract from a JSON string.
    pub fn from_json_str(input: &str) -> Result<Self> {
        crate::parser::parse_json(input)
    }

    /// Serialize this contract to a YAML string.
    pub fn to_yaml_str(&self) -> Result<String> {
        crate::parser::to_yaml(self)
    }

    /// Serialize this contract to a JSON string.
    pub fn to_json_str(&self) -> Result<String> {
        crate::parser::to_json(self)
    }

    /// Serialize this contract to a YAML file.
    pub fn to_yaml_file(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        crate::parser::to_yaml_file(self, path)
    }

    /// Serialize this contract to a JSON file.
    pub fn to_json_file(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        crate::parser::to_json_file(self, path)
    }

    /// Validate this contract and return a deterministic report.
    pub fn validate(&self) -> ValidationReport {
        validation::validate(self)
    }

    /// Clone this contract with reserved colliding extension keys removed for wire output.
    ///
    /// Validation still reports those keys via `DPCS-COM-012`; serialization omits them so
    /// YAML/JSON does not emit duplicate root keys.
    pub(crate) fn for_wire_serialization(&self) -> Self {
        let mut wire = self.clone();
        wire.extensions
            .retain(|key, _| !super::is_reserved_root_field(key));
        wire
    }

    /// Returns pipeline-level identity extracted from this contract.
    pub fn identity(&self) -> PipelineIdentity {
        PipelineIdentity {
            id: self.id.clone().into(),
            version: self.version.clone().into(),
            dpcs_version: self.dpcs_version.clone().into(),
            name: self.name.clone(),
        }
    }

    /// Builds a catalog of addressable objects within this contract.
    pub fn identity_catalog(&self) -> IdentityCatalog {
        IdentityCatalog::from_contract(self)
    }

    /// Returns the set of declared step identifiers.
    pub fn step_ids(&self) -> std::collections::BTreeSet<&str> {
        self.steps.iter().map(|step| step.id.as_str()).collect()
    }
}
