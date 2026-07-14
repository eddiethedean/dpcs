//! Capability profile types (SPEC Ch 16).

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::Path;

use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use crate::diagnostics::{categories, Diagnostic, DiagnosticStage, Severity, ValidationReport};
use crate::error::{Error, Result};
use crate::model::{ExtensionMap, Metadata};

/// Declared capability of an orchestrator profile.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityDecl {
    /// Stable capability identifier matched against plan requirements.
    pub id: String,
    /// Optional capability category (scheduling, execution, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Whether this capability is optional on the profile side.
    #[serde(default)]
    pub optional: bool,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

impl<'de> Deserialize<'de> for CapabilityDecl {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CapabilityDeclVisitor;

        impl<'de> Visitor<'de> for CapabilityDeclVisitor {
            type Value = CapabilityDecl;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a capability id string or capability object")
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(CapabilityDecl {
                    id: value.to_owned(),
                    category: None,
                    optional: false,
                    extensions: ExtensionMap::default(),
                })
            }

            fn visit_string<E>(self, value: String) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(CapabilityDecl {
                    id: value,
                    category: None,
                    optional: false,
                    extensions: ExtensionMap::default(),
                })
            }

            fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut id: Option<String> = None;
                let mut category: Option<String> = None;
                let mut optional = false;
                let mut extensions = ExtensionMap::default();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "id" => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        "category" => {
                            if category.is_some() {
                                return Err(de::Error::duplicate_field("category"));
                            }
                            category = Some(map.next_value()?);
                        }
                        "optional" => {
                            optional = map.next_value()?;
                        }
                        other => {
                            let value = map.next_value()?;
                            extensions.insert(other.to_owned(), value);
                        }
                    }
                }

                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                Ok(CapabilityDecl {
                    id,
                    category,
                    optional,
                    extensions,
                })
            }
        }

        deserializer.deserialize_any(CapabilityDeclVisitor)
    }
}

/// Capability profile for an orchestrator (SPEC Ch 16 §6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityProfile {
    /// Orchestrator identity.
    #[serde(alias = "profile")]
    pub identity: String,
    /// Supported DPCS specification version.
    ///
    /// Defaults to empty when omitted so legacy stub YAML can load; empty values
    /// fail [`validate_profile`] with `DPCS-CAP-004`.
    #[serde(default)]
    pub dpcs_version: String,
    /// Capabilities supported by this profile.
    ///
    /// Accepts either capability objects (`{ id: ... }`) or bare id strings.
    #[serde(default)]
    pub capabilities: Vec<CapabilityDecl>,
    /// Known limitations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limitations: Vec<String>,
    /// Optional descriptive metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Deprecated name alias for [`CapabilityProfile`].
///
/// Prefer [`CapabilityProfile`]. Field shape follows 0.7 (`identity`,
/// `dpcs_version`, `Vec<CapabilityDecl>`); wire YAML may still use `profile`
/// and bare capability id strings.
#[deprecated(note = "use CapabilityProfile")]
pub type OrchestratorCapabilities = CapabilityProfile;

impl CapabilityProfile {
    /// Create a minimal profile with an identity and supported DPCS version.
    pub fn new(identity: impl Into<String>, dpcs_version: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
            dpcs_version: dpcs_version.into(),
            capabilities: Vec::new(),
            limitations: Vec::new(),
            metadata: None,
            extensions: ExtensionMap::default(),
        }
    }

    /// Parse a capability profile from a YAML string.
    pub fn from_yaml_str(input: &str) -> Result<Self> {
        serde_yaml::from_str(input).map_err(|err| profile_parse_error(err.to_string()))
    }

    /// Parse a capability profile from a JSON string.
    pub fn from_json_str(input: &str) -> Result<Self> {
        serde_json::from_str(input).map_err(|err| profile_parse_error(err.to_string()))
    }

    /// Parse a capability profile from a YAML file.
    pub fn from_yaml_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_yaml_str(&contents).map_err(|err| with_source_path(err, path))
    }

    /// Parse a capability profile from a JSON file.
    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_json_str(&contents).map_err(|err| with_source_path(err, path))
    }

    /// Parse a capability profile from a file by extension.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase());
        match ext.as_deref() {
            Some("yaml") | Some("yml") => Self::from_yaml_file(path),
            Some("json") => Self::from_json_file(path),
            _ => Err(Error::UnsupportedFormat {
                path: path.to_path_buf(),
            }),
        }
    }

    /// Serialize this profile to a YAML string.
    pub fn to_yaml_str(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|err| Error::Serialization(format!("failed to serialize YAML: {err}")))
    }

    /// Serialize this profile to a JSON string.
    pub fn to_json_str(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|err| Error::Serialization(format!("failed to serialize JSON: {err}")))
    }
}

/// Validate profile declaration consistency (SPEC Ch 16 §8).
pub fn validate_profile(profile: &CapabilityProfile) -> ValidationReport {
    let mut report = ValidationReport::new();

    if profile.identity.trim().is_empty() {
        report.push(
            Diagnostic::capability_error(
                "DPCS-CAP-003",
                categories::CAPABILITY,
                "capability profile must declare a non-empty identity",
            )
            .with_object_ref("identity")
            .with_remediation("Set identity (or legacy profile) to the orchestrator name"),
        );
    }

    if profile.dpcs_version.trim().is_empty() {
        report.push(
            Diagnostic::capability_error(
                "DPCS-CAP-004",
                categories::CAPABILITY,
                "capability profile must declare a non-empty dpcsVersion",
            )
            .with_object_ref("dpcsVersion")
            .with_remediation("Set dpcsVersion to a supported DPCS specification version"),
        );
    }

    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for (index, capability) in profile.capabilities.iter().enumerate() {
        let object_ref = format!("capabilities[{index}]");
        if capability.id.trim().is_empty() {
            report.push(
                Diagnostic::capability_error(
                    "DPCS-CAP-001",
                    categories::CAPABILITY,
                    "capability id must not be empty",
                )
                .with_object_ref(format!("{object_ref}.id")),
            );
            continue;
        }

        let key = capability.id.trim().to_owned();
        if let Some(first_index) = seen.get(&key) {
            report.push(
                Diagnostic::capability_error(
                    "DPCS-CAP-002",
                    categories::CAPABILITY,
                    format!("duplicate capability id `{key}`"),
                )
                .with_object_ref(object_ref)
                .with_remediation(format!(
                    "Capability `{key}` was first declared at capabilities[{first_index}]"
                )),
            );
        } else {
            seen.insert(key, index);
        }
    }

    report.sort_deterministic();
    report
}

fn profile_parse_error(message: String) -> Error {
    let mut report = ValidationReport::new();
    report.push(Diagnostic {
        id: "DPCS-PARSE-002".to_owned(),
        severity: Severity::Error,
        stage: DiagnosticStage::Parse,
        category: categories::SYNTAX.to_owned(),
        message: format!("invalid capability profile: {message}"),
        object_ref: None,
        remediation: Some(
            "Provide identity (or profile), dpcsVersion, and capabilities[] with ids".to_owned(),
        ),
        source_location: None,
    });
    Error::InvalidDocument { report }
}

fn with_source_path(err: Error, path: &Path) -> Error {
    match err {
        Error::InvalidDocument { mut report } => {
            let location = path.display().to_string();
            for diagnostic in &mut report.diagnostics {
                if diagnostic.source_location.is_none() {
                    diagnostic.source_location = Some(location.clone());
                }
            }
            Error::InvalidDocument { report }
        }
        other => other,
    }
}
