//! Processing stages that emit diagnostics.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Stage of the DPCS processing pipeline that produced a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticStage {
    /// Document parsing.
    Parse,
    /// Canonical Object Model construction.
    CanonicalObjectModel,
    /// Semantic and structural validation.
    Validation,
    /// Compatibility analysis between artifacts.
    CompatibilityAnalysis,
    /// Pipeline plan generation.
    Planning,
    /// Orchestrator capability evaluation.
    CapabilityEvaluation,
    /// Orchestrator binding.
    OrchestratorBinding,
    /// Execution analysis (reserved; unused in 0.9.0 toolkit scope).
    ExecutionAnalysis,
}

impl fmt::Display for DiagnosticStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse => write!(f, "parse"),
            Self::CanonicalObjectModel => write!(f, "canonicalObjectModel"),
            Self::Validation => write!(f, "validation"),
            Self::CompatibilityAnalysis => write!(f, "compatibilityAnalysis"),
            Self::Planning => write!(f, "planning"),
            Self::CapabilityEvaluation => write!(f, "capabilityEvaluation"),
            Self::OrchestratorBinding => write!(f, "orchestratorBinding"),
            Self::ExecutionAnalysis => write!(f, "executionAnalysis"),
        }
    }
}
