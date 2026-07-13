//! Registry model skeleton.

use serde::{Deserialize, Serialize};

use super::ExtensionMap;

/// Reference to an external DPCS registry entry.
///
/// Skeleton for roadmap 0.9.0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
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
