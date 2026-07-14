//! Package manifest COM.

use serde::{Deserialize, Serialize};

use crate::model::ExtensionMap;
use crate::DPCS_SPEC_VERSION;

/// Top-level manifest for a `.dpcspkg` pipeline package.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct PackageManifest {
    /// Stable package identifier.
    pub id: String,
    /// Package version.
    pub version: String,
    /// Supported DPCS specification version.
    pub dpcs_version: String,
    /// Optional human-readable name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Indexed package artifacts.
    #[serde(default)]
    pub artifacts: Vec<PackageArtifactEntry>,
    /// Extension fields.
    #[serde(default, flatten)]
    #[cfg_attr(feature = "jsonschema", schemars(skip))]
    pub extensions: ExtensionMap,
}

impl Default for PackageManifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            version: "0.1.0".to_owned(),
            dpcs_version: DPCS_SPEC_VERSION.to_owned(),
            name: None,
            description: None,
            artifacts: Vec::new(),
            extensions: ExtensionMap::default(),
        }
    }
}

/// One artifact entry inside a package manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct PackageArtifactEntry {
    /// Artifact identifier (unique within the package).
    pub id: String,
    /// Artifact type (for example `pipelineContract` or `capabilityProfile`).
    #[serde(rename = "type")]
    pub artifact_type: String,
    /// Artifact version.
    pub version: String,
    /// Path relative to the package root (POSIX separators).
    pub path: String,
    /// Extension fields.
    #[serde(default, flatten)]
    #[cfg_attr(feature = "jsonschema", schemars(skip))]
    pub extensions: ExtensionMap,
}
