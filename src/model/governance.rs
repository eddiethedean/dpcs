//! Governance metadata (SPEC Ch 25).

use serde::{Deserialize, Serialize};

use super::ExtensionMap;

/// Declarative governance metadata attached to a Pipeline Contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceMetadata {
    /// Owning team or organization (may mirror `metadata.owner`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Governing authority when distinct from owner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governing_authority: Option<String>,
    /// Publication status (for example `draft`, `published`, `deprecated`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publication_status: Option<String>,
    /// Publication timestamp (implementation-defined string, typically ISO-8601).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    /// Stewardship or classification tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Known publication statuses used by validators.
pub const PUBLICATION_STATUSES: &[&str] =
    &["draft", "published", "deprecated", "retired", "unpublished"];

/// Returns `true` when `status` is a recognized publication status.
pub fn is_known_publication_status(status: &str) -> bool {
    PUBLICATION_STATUSES
        .iter()
        .any(|known| known.eq_ignore_ascii_case(status.trim()))
}
