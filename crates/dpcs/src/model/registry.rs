//! Registry model (SPEC Ch 22).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{
    is_known_publication_status, is_present_version, is_valid_version, ExtensionMap,
    GovernanceMetadata, SecurityMetadata,
};
use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::error::{Error, Result};
use crate::DPCS_SPEC_VERSION;

/// Reference to an external DPCS registry entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct RegistryReference {
    /// Registry identifier or URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
    /// Artifact identifier within the registry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// In-process DPCS registry document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Registry {
    /// Stable registry identifier.
    pub id: String,
    /// Registry version.
    pub version: String,
    /// Supported DPCS specification version.
    pub dpcs_version: String,
    /// Ownership / governing authority.
    pub owner: String,
    /// Publication metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publication_status: Option<String>,
    /// Optional published-at timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    /// Optional governance block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance: Option<GovernanceMetadata>,
    /// Optional security block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityMetadata>,
    /// Registered artifacts.
    #[serde(default)]
    pub artifacts: Vec<RegisteredArtifact>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// A single registered artifact entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct RegisteredArtifact {
    /// Unique artifact identifier within the registry.
    pub id: String,
    /// Artifact type (for example `pipelineContract` or `capabilityProfile`).
    #[serde(rename = "type")]
    pub artifact_type: String,
    /// Artifact version.
    pub version: String,
    /// Optional compatibility metadata summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<String>,
    /// Publication status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publication_status: Option<String>,
    /// Optional location / payload reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

impl Registry {
    /// Parse a registry document from a YAML or JSON file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path).map_err(|err| Error::Io {
            path: PathBuf::from(path),
            source: err,
        })?;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let result = if matches!(ext.as_str(), "json") {
            Self::from_json_str(&contents)
        } else {
            Self::from_yaml_str(&contents)
        };
        match result {
            Ok(registry) => Ok(registry),
            Err(Error::InvalidDocument { mut report }) => {
                let location = path.display().to_string();
                for diagnostic in &mut report.diagnostics {
                    if diagnostic.source_location.is_none() {
                        diagnostic.source_location = Some(location.clone());
                    }
                }
                Err(Error::InvalidDocument { report })
            }
            Err(err) => Err(err),
        }
    }

    /// Parse a registry from YAML.
    pub fn from_yaml_str(input: &str) -> Result<Self> {
        serde_yaml::from_str(input).map_err(|err| registry_parse_error(err.to_string()))
    }

    /// Parse a registry from JSON.
    pub fn from_json_str(input: &str) -> Result<Self> {
        serde_json::from_str(input).map_err(|err| registry_parse_error(err.to_string()))
    }
}

/// Validate a registry document.
pub fn validate_registry(registry: &Registry) -> ValidationReport {
    let mut report = ValidationReport::new();

    if registry.id.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-REG-001",
                categories::REGISTRY,
                "registry id must not be empty",
            )
            .with_object_ref("id"),
        );
    }
    if !is_present_version(&registry.version) {
        report.push(
            Diagnostic::error(
                "DPCS-REG-002",
                categories::REGISTRY,
                "registry version must not be empty",
            )
            .with_object_ref("version"),
        );
    } else if !is_valid_version(&registry.version) {
        report.push(
            Diagnostic::error(
                "DPCS-REG-003",
                categories::REGISTRY,
                format!(
                    "registry version `{}` is not SemVer-compatible",
                    registry.version
                ),
            )
            .with_object_ref("version"),
        );
    }
    if registry.dpcs_version.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-REG-004",
                categories::REGISTRY,
                "registry must declare dpcsVersion",
            )
            .with_object_ref("dpcsVersion"),
        );
    } else if !is_valid_version(&registry.dpcs_version) {
        report.push(
            Diagnostic::error(
                "DPCS-REG-005",
                categories::REGISTRY,
                format!(
                    "registry dpcsVersion `{}` is not SemVer-compatible",
                    registry.dpcs_version
                ),
            )
            .with_object_ref("dpcsVersion"),
        );
    } else if registry.dpcs_version != DPCS_SPEC_VERSION && !registry.dpcs_version.starts_with("1.")
    {
        report.push(
            Diagnostic::warning(
                "DPCS-REG-006",
                categories::REGISTRY,
                format!(
                    "registry dpcsVersion `{}` may not be supported",
                    registry.dpcs_version
                ),
            )
            .with_object_ref("dpcsVersion"),
        );
    }
    if registry.owner.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-REG-007",
                categories::REGISTRY,
                "registry must declare an owner",
            )
            .with_object_ref("owner"),
        );
    }
    if let Some(status) = &registry.publication_status {
        if status.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-REG-015",
                    categories::REGISTRY,
                    "registry publicationStatus must not be empty when declared",
                )
                .with_object_ref("publicationStatus"),
            );
        } else if !is_known_publication_status(status) {
            report.push(
                Diagnostic::warning(
                    "DPCS-REG-008",
                    categories::REGISTRY,
                    format!("registry publicationStatus `{status}` is not recognized"),
                )
                .with_object_ref("publicationStatus"),
            );
        }
    }

    let mut seen: BTreeMap<(String, String), usize> = BTreeMap::new();
    for (index, artifact) in registry.artifacts.iter().enumerate() {
        let object_ref = format!("artifacts[{index}]");
        if artifact.id.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-REG-009",
                    categories::REGISTRY,
                    "registered artifact id must not be empty",
                )
                .with_object_ref(format!("{object_ref}.id")),
            );
            continue;
        }
        if artifact.artifact_type.trim().is_empty() {
            report.push(
                Diagnostic::error(
                    "DPCS-REG-010",
                    categories::REGISTRY,
                    "registered artifact type must not be empty",
                )
                .with_object_ref(format!("{object_ref}.type")),
            );
        }
        if !is_present_version(&artifact.version) {
            report.push(
                Diagnostic::error(
                    "DPCS-REG-011",
                    categories::REGISTRY,
                    "registered artifact version must not be empty",
                )
                .with_object_ref(format!("{object_ref}.version")),
            );
        } else if !is_valid_version(&artifact.version) {
            report.push(
                Diagnostic::error(
                    "DPCS-REG-012",
                    categories::REGISTRY,
                    format!(
                        "registered artifact version `{}` is not SemVer-compatible",
                        artifact.version
                    ),
                )
                .with_object_ref(format!("{object_ref}.version")),
            );
        }
        if let Some(status) = &artifact.publication_status {
            if !is_known_publication_status(status) {
                report.push(
                    Diagnostic::warning(
                        "DPCS-REG-013",
                        categories::REGISTRY,
                        format!("artifact publicationStatus `{status}` is not recognized"),
                    )
                    .with_object_ref(format!("{object_ref}.publicationStatus")),
                );
            }
        }

        let key = (artifact.id.clone(), artifact.version.clone());
        if let Some(first) = seen.get(&key).copied() {
            report.push(
                Diagnostic::error(
                    "DPCS-REG-014",
                    categories::REGISTRY,
                    format!(
                        "duplicate registered artifact `{}@{}`",
                        artifact.id, artifact.version
                    ),
                )
                .with_object_ref(object_ref)
                .with_remediation(format!("Artifact was first declared at artifacts[{first}]")),
            );
        } else {
            seen.insert(key, index);
        }
    }

    report.sort_deterministic();
    report
}

fn registry_parse_error(message: String) -> Error {
    let mut report = ValidationReport::new();
    report.push(
        Diagnostic::parse_error(
            "DPCS-PARSE-002",
            format!("invalid registry document: {message}"),
        )
        .with_remediation(
            "Provide id, version, dpcsVersion, owner, and artifacts[] with unique id+version",
        ),
    );
    Error::InvalidDocument { report }
}
