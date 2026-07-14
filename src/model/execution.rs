//! Execution requirements model (SPEC Ch 10).

use serde::{Deserialize, Serialize};

use super::ExtensionMap;

/// Declared execution requirements for a Pipeline Contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionRequirements {
    /// Required runtime capabilities (logical identifiers).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_capabilities: Vec<String>,
    /// Logical resource requirements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceRequirements>,
    /// Execution environment constraints.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<ExecutionEnvironment>,
    /// Isolation requirements (e.g. `process`, `container`, `vm`, `network`, `securityDomain`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub isolation: Vec<String>,
    /// External service dependencies required for execution.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_dependencies: Vec<ExternalDependency>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Logical resource declarations independent of vendor infrastructure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResourceRequirements {
    /// Processor requirements (logical quantity or constraint).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processor: Option<String>,
    /// Memory requirements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
    /// Storage capacity requirements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage: Option<String>,
    /// Accelerator requirements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accelerator: Option<String>,
    /// Bandwidth expectations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bandwidth: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Characteristics of the execution environment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionEnvironment {
    /// Operating system constraints.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operating_system: Option<String>,
    /// Runtime dependencies.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runtime_dependencies: Vec<String>,
    /// Software capabilities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub software_capabilities: Vec<String>,
    /// Container requirements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
    /// Execution profile identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// External service dependency required for successful execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDependency {
    /// Logical service identity.
    pub id: String,
    /// Required capability of the service.
    pub capability: String,
    /// Availability expectations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub availability: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}
