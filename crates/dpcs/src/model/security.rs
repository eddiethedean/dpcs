//! Security metadata (SPEC Ch 24).

use serde::{Deserialize, Serialize};

use super::ExtensionMap;

/// Declarative security metadata attached to a Pipeline Contract.
///
/// Authentication, authorization, and secret-management systems remain outside
/// DPCS scope. This model records portable references and integrity pointers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SecurityMetadata {
    /// External secret references (URIs or vault paths). Must not embed secret values.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secret_refs: Vec<SecretReference>,
    /// Integrity references (hashes, signatures, or attestation pointers).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub integrity_refs: Vec<IntegrityReference>,
    /// Optional security domain / isolation annotation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security_domain: Option<String>,
    /// Optional audit policy identifier or reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_policy_ref: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Reference to an externally managed secret.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SecretReference {
    /// Stable identifier for this secret reference.
    pub id: String,
    /// External locator (URI, vault path, or provider reference).
    pub loc: String,
    /// Optional provider hint (for example `vault` or `aws-secrets-manager`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Integrity reference for tamper detection / attestation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct IntegrityReference {
    /// Algorithm or kind (for example `sha256` or `cosign`).
    pub kind: String,
    /// Digest, signature reference, or external locator (not an embedded key).
    pub value: String,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}
