//! Extension helpers and extension definition COM (SPEC Ch 21).

use serde::{Deserialize, Serialize};

use super::ExtensionMap;

/// Declared extension definition (standalone document or registry artifact).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionDefinition {
    /// Stable extension identifier.
    pub id: String,
    /// Unique extension namespace.
    pub namespace: String,
    /// Extension version.
    pub version: String,
    /// Owner or governing authority.
    pub owner: String,
    /// Declared extension scope.
    pub scope: String,
    /// Human-readable semantics summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantics: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Returns `true` when `key` is a reserved DPCS root field name.
pub fn is_reserved_root_field(key: &str) -> bool {
    matches!(
        key,
        "dpcsVersion"
            | "id"
            | "name"
            | "version"
            | "metadata"
            | "interface"
            | "graph"
            | "steps"
            | "contractReferences"
            | "dataFlow"
            | "controlFlow"
            | "execution"
            | "scheduling"
            | "qualityGates"
            | "failureSemantics"
            | "lineage"
            | "compatibility"
            | "security"
            | "governance"
    )
}

/// Returns `true` when an extension field / namespace key is syntactically valid.
///
/// Accepted forms:
/// - `x-*` vendor prefix
/// - namespaced keys containing `:` (for example `acme:feature`)
/// - URI-like keys containing `://`
pub fn is_valid_extension_namespace(key: &str) -> bool {
    let key = key.trim();
    if key.is_empty() {
        return false;
    }
    if key.starts_with("x-") || key.starts_with("X-") {
        return key.len() > 2
            && key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    }
    if key.contains("://") {
        return key.split("://").nth(1).is_some_and(|rest| !rest.is_empty());
    }
    if let Some((ns, local)) = key.split_once(':') {
        return !ns.is_empty()
            && !local.is_empty()
            && ns
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
            && local
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    }
    false
}
