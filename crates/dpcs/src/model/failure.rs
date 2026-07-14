//! Failure semantics model (SPEC Ch 13).

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{ExtensionMap, Metadata};

/// Declared failure behavior for a pipeline scope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct FailureSemantics {
    /// Stable failure-semantics identifier.
    pub id: String,
    /// Scope to which the semantics apply.
    pub scope: FailureScope,
    /// Conditions that trigger this failure handling.
    pub triggers: Vec<String>,
    /// Required responses when triggered.
    pub responses: Vec<FailureResponse>,
    /// Optional failure category.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Optional retry semantics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetrySemantics>,
    /// Optional compensation description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compensation: Option<String>,
    /// Optional recovery semantics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery: Option<RecoverySemantics>,
    /// Optional descriptive metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Scope of a failure semantics declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct FailureScope {
    /// Scope kind (`pipeline`, `step`, `path`, or extension).
    pub kind: String,
    /// Target step identifier when kind is `step`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
    /// Logical path when kind is `path`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl FailureScope {
    /// Returns whether this scope targets a pipeline step.
    pub fn is_step(&self) -> bool {
        self.kind.eq_ignore_ascii_case("step")
    }

    /// Pipeline-wide scope.
    pub fn pipeline() -> Self {
        Self {
            kind: "pipeline".to_owned(),
            step_id: None,
            path: None,
        }
    }
}

/// Logical failure response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureResponse {
    /// Abort execution.
    Abort,
    /// Retry failed work.
    Retry,
    /// Compensate previously completed work.
    Compensate,
    /// Continue despite the failure.
    Continue,
    /// Skip the failed unit of work.
    Skip,
    /// Route to an alternate execution path.
    AlternatePath,
    /// Require manual approval.
    RequireApproval,
    /// Emit diagnostics.
    EmitDiagnostics,
    /// Implementation-defined response.
    Extension(String),
}

impl FailureResponse {
    /// Returns the wire-form response string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Abort => "abort",
            Self::Retry => "retry",
            Self::Compensate => "compensate",
            Self::Continue => "continue",
            Self::Skip => "skip",
            Self::AlternatePath => "alternatePath",
            Self::RequireApproval => "requireApproval",
            Self::EmitDiagnostics => "emitDiagnostics",
            Self::Extension(value) => value.as_str(),
        }
    }

    /// Returns whether this response is retry.
    pub fn is_retry(&self) -> bool {
        matches!(self, Self::Retry)
    }
}

impl Serialize for FailureResponse {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for FailureResponse {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "abort" => Self::Abort,
            "retry" => Self::Retry,
            "compensate" => Self::Compensate,
            "continue" => Self::Continue,
            "skip" => Self::Skip,
            "alternatePath" => Self::AlternatePath,
            "requireApproval" => Self::RequireApproval,
            "emitDiagnostics" => Self::EmitDiagnostics,
            other => Self::Extension(other.to_owned()),
        })
    }
}

#[cfg(feature = "jsonschema")]
impl schemars::JsonSchema for FailureResponse {
    fn schema_name() -> String {
        "FailureResponse".to_owned()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        String::json_schema(gen)
    }
}

/// Retry behavior for failure semantics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct RetrySemantics {
    /// Whether retry is eligible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eligible: Option<bool>,
    /// Maximum retry attempts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_attempts: Option<u32>,
    /// Conditions under which retry applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conditions: Option<String>,
    /// Delay policy between attempts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay_policy: Option<String>,
    /// Termination conditions for retry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub termination: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Recovery behavior for failure semantics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct RecoverySemantics {
    /// Restart behavior.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restart: Option<String>,
    /// Resume behavior.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume: Option<String>,
    /// Rollback expectations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback: Option<String>,
    /// Checkpoint usage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint: Option<String>,
    /// State restoration requirements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_restoration: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}
