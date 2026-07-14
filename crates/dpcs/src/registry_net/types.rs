//! Shared registry network types (client + server).

use serde::{Deserialize, Serialize};

use crate::model::RegisteredArtifact;

/// Publish / update request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishRequest {
    /// Artifact metadata (id in path must match `artifact.id` when present).
    pub artifact: RegisteredArtifact,
    /// Optional UTF-8 payload content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Content encoding (`utf-8` default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_encoding: Option<String>,
}
