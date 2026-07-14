//! Root Pipeline Contract object.

use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};

use super::{
    is_reserved_root_field, CompatibilityPolicy, ContractReference, ControlFlow, DataFlow,
    ExecutionRequirements, ExtensionMap, ExtensionValue, FailureSemantics, GovernanceMetadata,
    IdentityCatalog, Metadata, PipelineGraph, PipelineIdentity, PipelineInterface, PipelineLineage,
    PipelineStep, QualityGate, SchedulingIntent, SecurityMetadata,
};
use crate::diagnostics::ValidationReport;
use crate::error::Result;
use crate::validation;

/// Root Canonical Object Model for a DPCS Pipeline Contract.
///
/// Field names follow the camelCase serialization used in `SPEC.md` examples.
///
/// Wire serialization omits reserved root keys that collide in `extensions`
/// (without cloning the contract) so YAML/JSON does not emit duplicate keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
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
    /// Optional security metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityMetadata>,
    /// Optional governance metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance: Option<GovernanceMetadata>,
    /// Extension fields preserved from the source document.
    ///
    /// Reserved colliding keys are omitted on serialize (validation still reports them).
    #[serde(default, flatten, serialize_with = "serialize_root_extensions")]
    pub extensions: ExtensionMap,
}

/// Serialize root extensions while omitting reserved colliding keys.
fn serialize_root_extensions<S>(
    extensions: &ExtensionMap,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let filtered: Vec<(&String, &ExtensionValue)> = extensions
        .iter()
        .filter(|(key, _)| !is_reserved_root_field(key))
        .collect();
    let mut map = serializer.serialize_map(Some(filtered.len()))?;
    for (key, value) in filtered {
        map.serialize_entry(key, value)?;
    }
    map.end()
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
