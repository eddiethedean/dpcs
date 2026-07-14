//! Compatibility policy model (SPEC Ch 19).

use serde::{Deserialize, Serialize};

use super::ExtensionMap;

/// Declared compatibility policy for a Pipeline Contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityPolicy {
    /// Compatibility mode (for example `backward` or `full`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}
