//! Pipeline metadata.
//!
//! DPCS names metadata as a root and interface slot (SPEC Ch 3 §4, Ch 4 §3).
//! This crate provides an initial metadata profile with common descriptive
//! fields. Additional metadata MAY be supplied through extension fields.

use serde::{Deserialize, Serialize};

use super::ExtensionMap;

/// Descriptive metadata attached to a Pipeline Contract or interface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// Free-form description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owning team or organization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Classification tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Extension metadata.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}
