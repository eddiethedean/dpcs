//! Orchestrator capability profile types.

use serde::{Deserialize, Serialize};

/// Declared capabilities of an orchestrator or execution profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OrchestratorCapabilities {
    /// Profile name (for example `airflow`, `dagster`, `prefect`).
    pub profile: String,
    /// Capability identifiers supported by the profile.
    #[serde(default)]
    pub capabilities: Vec<String>,
}

impl OrchestratorCapabilities {
    /// Create a capability profile.
    pub fn new(profile: impl Into<String>) -> Self {
        Self {
            profile: profile.into(),
            capabilities: Vec::new(),
        }
    }
}
