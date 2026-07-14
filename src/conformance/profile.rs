//! Conformance profile and claim validation (SPEC Ch 23).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::error::{Error, Result};
use crate::model::{is_valid_version, ExtensionMap};
use crate::{DPCS_SPEC_VERSION, VERSION};

/// Conformance levels defined by SPEC Ch 23.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConformanceLevel {
    /// Parser conformance.
    Parser,
    /// Validator conformance.
    Validator,
    /// Planner conformance.
    Planner,
    /// Capability evaluator conformance.
    CapabilityEvaluator,
    /// Orchestrator binder conformance.
    OrchestratorBinder,
    /// Registry document conformance.
    Registry,
    /// Complete implementation conformance.
    CompleteImplementation,
}

impl ConformanceLevel {
    /// Wire / display name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Parser => "parser",
            Self::Validator => "validator",
            Self::Planner => "planner",
            Self::CapabilityEvaluator => "capabilityEvaluator",
            Self::OrchestratorBinder => "orchestratorBinder",
            Self::Registry => "registry",
            Self::CompleteImplementation => "completeImplementation",
        }
    }
}

impl std::fmt::Display for ConformanceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Levels implemented by this `dpcs` toolkit.
pub fn implemented_levels() -> &'static [ConformanceLevel] {
    &[
        ConformanceLevel::Parser,
        ConformanceLevel::Validator,
        ConformanceLevel::Planner,
        ConformanceLevel::CapabilityEvaluator,
        ConformanceLevel::OrchestratorBinder,
        ConformanceLevel::Registry,
        ConformanceLevel::CompleteImplementation,
    ]
}

/// Conformance profile document constraining validation / claims.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConformanceProfile {
    /// Profile identity.
    pub id: String,
    /// Profile version.
    pub version: String,
    /// Supported DPCS specification version.
    pub dpcs_version: String,
    /// Claimed conformance levels.
    #[serde(default)]
    pub levels: Vec<ConformanceLevel>,
    /// Extension namespaces forbidden on contracts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forbidden_extension_namespaces: Vec<String>,
    /// When true, contracts must declare a `security` block.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub require_security: bool,
    /// When true, contracts must declare a `governance` block.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub require_governance: bool,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Explicit conformance claim for an implementation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConformanceClaim {
    /// Implementation name.
    pub implementation: String,
    /// Implementation version.
    pub implementation_version: String,
    /// Claimed DPCS specification version.
    pub dpcs_version: String,
    /// Claimed levels.
    pub levels: Vec<ConformanceLevel>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

impl ConformanceProfile {
    /// Parse from a YAML or JSON file.
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
            Ok(profile) => Ok(profile),
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

    /// Parse from YAML.
    pub fn from_yaml_str(input: &str) -> Result<Self> {
        serde_yaml::from_str(input).map_err(|err| profile_parse_error(err.to_string()))
    }

    /// Parse from JSON.
    pub fn from_json_str(input: &str) -> Result<Self> {
        serde_json::from_str(input).map_err(|err| profile_parse_error(err.to_string()))
    }
}

/// Default claim for this toolkit build.
pub fn toolkit_claim() -> ConformanceClaim {
    ConformanceClaim {
        implementation: "dpcs".to_owned(),
        implementation_version: VERSION.to_owned(),
        dpcs_version: DPCS_SPEC_VERSION.to_owned(),
        levels: implemented_levels().to_vec(),
        extensions: ExtensionMap::new(),
    }
}

/// Validate a conformance profile document.
pub fn validate_profile(profile: &ConformanceProfile) -> ValidationReport {
    let mut report = ValidationReport::new();

    if profile.id.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-CONF-001",
                categories::CONFORMANCE,
                "conformance profile id must not be empty",
            )
            .with_object_ref("id"),
        );
    }
    if profile.version.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-CONF-002",
                categories::CONFORMANCE,
                "conformance profile version must not be empty",
            )
            .with_object_ref("version"),
        );
    } else if !is_valid_version(&profile.version) {
        report.push(
            Diagnostic::error(
                "DPCS-CONF-003",
                categories::CONFORMANCE,
                format!(
                    "conformance profile version `{}` is not SemVer-compatible",
                    profile.version
                ),
            )
            .with_object_ref("version"),
        );
    }
    if profile.dpcs_version.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-CONF-004",
                categories::CONFORMANCE,
                "conformance profile must declare dpcsVersion",
            )
            .with_object_ref("dpcsVersion"),
        );
    } else if !is_valid_version(&profile.dpcs_version) {
        report.push(
            Diagnostic::error(
                "DPCS-CONF-005",
                categories::CONFORMANCE,
                format!(
                    "conformance profile dpcsVersion `{}` is not SemVer-compatible",
                    profile.dpcs_version
                ),
            )
            .with_object_ref("dpcsVersion"),
        );
    } else if !crate::model::versions_compatible(&profile.dpcs_version, DPCS_SPEC_VERSION) {
        report.push(
            Diagnostic::error(
                "DPCS-CONF-009",
                categories::CONFORMANCE,
                format!(
                    "conformance profile dpcsVersion `{}` is incompatible with toolkit `{DPCS_SPEC_VERSION}`",
                    profile.dpcs_version
                ),
            )
            .with_object_ref("dpcsVersion"),
        );
    }

    let supported: std::collections::BTreeSet<_> = implemented_levels().iter().copied().collect();
    for level in &profile.levels {
        if !supported.contains(level) {
            report.push(
                Diagnostic::error(
                    "DPCS-CONF-010",
                    categories::CONFORMANCE,
                    format!("conformance level `{level}` is not implemented by this toolkit"),
                )
                .with_object_ref("levels"),
            );
        }
    }
    if profile.levels.is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-CONF-006",
                categories::CONFORMANCE,
                "conformance profile must claim at least one level",
            )
            .with_object_ref("levels"),
        );
    }

    report.sort_deterministic();
    report
}

/// Validate a conformance claim against this toolkit.
pub fn validate_claim(claim: &ConformanceClaim) -> ValidationReport {
    let mut report = ValidationReport::new();

    if claim.implementation.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-CONF-007",
                categories::CONFORMANCE,
                "conformance claim implementation must not be empty",
            )
            .with_object_ref("implementation"),
        );
    }
    if claim.dpcs_version.trim().is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-CONF-008",
                categories::CONFORMANCE,
                "conformance claim must declare dpcsVersion",
            )
            .with_object_ref("dpcsVersion"),
        );
    } else if !crate::model::versions_compatible(&claim.dpcs_version, DPCS_SPEC_VERSION) {
        report.push(
            Diagnostic::error(
                "DPCS-CONF-009",
                categories::CONFORMANCE,
                format!(
                    "conformance claim dpcsVersion `{}` is incompatible with toolkit `{DPCS_SPEC_VERSION}`",
                    claim.dpcs_version
                ),
            )
            .with_object_ref("dpcsVersion"),
        );
    }

    let supported: std::collections::BTreeSet<_> = implemented_levels().iter().copied().collect();
    for level in &claim.levels {
        if !supported.contains(level) {
            report.push(
                Diagnostic::error(
                    "DPCS-CONF-010",
                    categories::CONFORMANCE,
                    format!("conformance level `{level}` is not implemented by this toolkit"),
                )
                .with_object_ref("levels"),
            );
        }
    }
    if claim.levels.is_empty() {
        report.push(
            Diagnostic::error(
                "DPCS-CONF-011",
                categories::CONFORMANCE,
                "conformance claim must include at least one level",
            )
            .with_object_ref("levels"),
        );
    }

    report.sort_deterministic();
    report
}

/// Apply profile requirements to a contract validation report (extra checks).
pub fn apply_profile_to_contract(
    contract: &crate::model::PipelineContract,
    profile: &ConformanceProfile,
) -> ValidationReport {
    let mut report = ValidationReport::new();

    if profile.require_security && contract.security.is_none() {
        report.push(
            Diagnostic::error(
                "DPCS-CONF-012",
                categories::CONFORMANCE,
                "conformance profile requires a security block",
            )
            .with_object_ref("security")
            .with_remediation("Declare security metadata on the Pipeline Contract"),
        );
    }
    if profile.require_governance && contract.governance.is_none() {
        report.push(
            Diagnostic::error(
                "DPCS-CONF-013",
                categories::CONFORMANCE,
                "conformance profile requires a governance block",
            )
            .with_object_ref("governance")
            .with_remediation("Declare governance metadata on the Pipeline Contract"),
        );
    }

    if !profile.forbidden_extension_namespaces.is_empty() {
        let options = crate::validation::ExtensionValidationOptions {
            forbidden_namespaces: profile.forbidden_extension_namespaces.clone(),
        };
        // Only keep EXT-002 findings from forbidden namespaces; avoid duplicate EXT-010 noise.
        let ext = crate::validation::validate_extensions_with_options(contract, &options);
        for diagnostic in ext.diagnostics {
            if diagnostic.id == "DPCS-EXT-002" {
                report.push(diagnostic);
            }
        }
    }

    report.sort_deterministic();
    report
}

fn profile_parse_error(message: String) -> Error {
    let mut report = ValidationReport::new();
    report.push(
        Diagnostic::parse_error(
            "DPCS-PARSE-002",
            format!("invalid conformance profile: {message}"),
        )
        .with_remediation("Provide id, version, dpcsVersion, and levels[]"),
    );
    Error::InvalidDocument { report }
}
