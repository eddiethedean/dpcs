//! Scheduling intent model.

use serde::{Deserialize, Serialize};

use super::ExtensionMap;

/// Declared scheduling intent for a Pipeline Contract.
///
/// Skeleton for roadmap 0.6.0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SchedulingIntent {
    /// Optional cron-like schedule expression.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,
    /// Optional timezone identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}
