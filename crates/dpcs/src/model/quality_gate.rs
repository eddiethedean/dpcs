//! Quality gate model (SPEC Ch 12).

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{ExtensionMap, Metadata};

/// Quality gate attached to a pipeline, step, or interface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct QualityGate {
    /// Stable quality-gate identifier.
    pub id: String,
    /// Logical purpose of the gate.
    pub purpose: String,
    /// One or more declarative evaluation criteria.
    pub criteria: Vec<QualityCriterion>,
    /// Behavior on successful evaluation.
    pub on_success: GateOutcome,
    /// Behavior on unsuccessful evaluation.
    pub on_failure: GateOutcome,
    /// Optional gate category.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Placement of the gate within the pipeline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement: Option<GatePlacement>,
    /// Optional descriptive metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Declarative evaluation criterion for a quality gate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct QualityCriterion {
    /// Optional criterion identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Criterion type (for example `odcs`, `dtcs`, `expression`).
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "type")]
    pub criterion_type: Option<String>,
    /// Reference to a contract in `contractReferences`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_ref: Option<String>,
    /// Declarative expression or rule identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expression: Option<String>,
    /// Extension fields.
    #[serde(default, flatten)]
    pub extensions: ExtensionMap,
}

/// Declared outcome of gate evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateOutcome {
    /// Abort pipeline execution.
    Abort,
    /// Retry evaluation or upstream work.
    Retry,
    /// Route to an alternate path.
    AlternatePath,
    /// Request approval before continuing.
    RequireApproval,
    /// Record diagnostics and continue according to policy.
    EmitDiagnostics,
    /// Continue pipeline execution.
    Continue,
    /// Implementation-defined outcome.
    Extension(String),
}

impl GateOutcome {
    /// Returns the wire-form outcome string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Abort => "abort",
            Self::Retry => "retry",
            Self::AlternatePath => "alternatePath",
            Self::RequireApproval => "requireApproval",
            Self::EmitDiagnostics => "emitDiagnostics",
            Self::Continue => "continue",
            Self::Extension(value) => value.as_str(),
        }
    }
}

impl Serialize for GateOutcome {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for GateOutcome {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "abort" => Self::Abort,
            "retry" => Self::Retry,
            "alternatePath" => Self::AlternatePath,
            "requireApproval" => Self::RequireApproval,
            "emitDiagnostics" => Self::EmitDiagnostics,
            "continue" => Self::Continue,
            other => Self::Extension(other.to_owned()),
        })
    }
}

#[cfg(feature = "jsonschema")]
impl schemars::JsonSchema for GateOutcome {
    fn schema_name() -> String {
        "GateOutcome".to_owned()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        String::json_schema(gen)
    }
}

/// Placement of a quality gate within a pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GatePlacement {
    /// Placement kind (`beforePipeline`, `beforeStep`, `afterStep`, `beforeCompletion`, or extension).
    pub kind: String,
    /// Target step identifier when kind is `beforeStep` or `afterStep`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
}

impl GatePlacement {
    /// Returns whether this placement targets a step.
    pub fn targets_step(&self) -> bool {
        let kind = self.kind.trim();
        kind.eq_ignore_ascii_case("beforeStep") || kind.eq_ignore_ascii_case("afterStep")
    }
}
