//! Execution requirements model.

use serde::{Deserialize, Serialize};

use super::ExtensionMap;

/// Declared execution requirements for a Pipeline Contract.
///
/// This is a skeleton for roadmap 0.6.0. Fields will expand as SPEC chapters
/// 10--15 are implemented.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionRequirements {
    /// Required runtime capabilities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_capabilities: Vec<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}
