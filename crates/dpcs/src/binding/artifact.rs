//! Binding targets and generated artifact types (SPEC Ch 17).

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::capabilities::CapabilityReport;

/// Supported orchestrator binding targets (ROADMAP 0.8.0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum BindingTarget {
    /// Apache Airflow DAG scaffold.
    Airflow,
    /// Dagster Definitions / job scaffold.
    Dagster,
    /// Prefect flow scaffold.
    Prefect,
    /// Temporal workflow stub (experimental).
    Temporal,
    /// Kubernetes Job / CronJob manifests (experimental).
    Kubernetes,
}

impl BindingTarget {
    /// Wire-form / CLI name for this target.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Airflow => "airflow",
            Self::Dagster => "dagster",
            Self::Prefect => "prefect",
            Self::Temporal => "temporal",
            Self::Kubernetes => "kubernetes",
        }
    }

    /// Returns whether this target is documented as experimental.
    pub fn is_experimental(self) -> bool {
        matches!(self, Self::Temporal | Self::Kubernetes)
    }

    /// All supported binding targets in stable order.
    pub fn all() -> &'static [BindingTarget] {
        &[
            Self::Airflow,
            Self::Dagster,
            Self::Prefect,
            Self::Temporal,
            Self::Kubernetes,
        ]
    }
}

impl fmt::Display for BindingTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for BindingTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "airflow" => Ok(Self::Airflow),
            "dagster" => Ok(Self::Dagster),
            "prefect" => Ok(Self::Prefect),
            "temporal" => Ok(Self::Temporal),
            "kubernetes" | "k8s" => Ok(Self::Kubernetes),
            other => Err(format!(
                "unknown binding target `{other}`; expected one of: airflow, dagster, prefect, temporal, kubernetes"
            )),
        }
    }
}

/// A single generated orchestration artifact file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BindingFile {
    /// Path relative to the binding output root (POSIX-style separators).
    pub relative_path: String,
    /// IANA media type (for example `text/x-python` or `application/yaml`).
    pub media_type: String,
    /// Full file contents.
    pub content: String,
}

impl BindingFile {
    /// Construct a binding file.
    pub fn new(
        relative_path: impl Into<String>,
        media_type: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            relative_path: relative_path.into(),
            media_type: media_type.into(),
            content: content.into(),
        }
    }
}

/// Bundle of platform-specific artifacts produced by a successful bind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BindingBundle {
    /// Target orchestrator adapter.
    pub target: BindingTarget,
    /// Source pipeline contract identifier.
    pub contract_id: String,
    /// Source pipeline contract version.
    pub contract_version: String,
    /// Capability profile identity used for the binding gate.
    pub profile_identity: String,
    /// Generated artifact files.
    pub files: Vec<BindingFile>,
    /// Capability match report from the pre-binding evaluation.
    pub capability: CapabilityReport,
}
