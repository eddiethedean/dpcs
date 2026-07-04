//! Processing stages that emit diagnostics.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Stage of the DPCS processing pipeline that produced a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticStage {
    /// Document parsing.
    Parse,
    /// Canonical Object Model construction.
    CanonicalObjectModel,
    /// Semantic and structural validation.
    Validation,
    /// Pipeline plan generation.
    Planning,
    /// Orchestrator capability evaluation.
    CapabilityEvaluation,
    /// Orchestrator binding.
    OrchestratorBinding,
}

impl fmt::Display for DiagnosticStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse => write!(f, "parse"),
            Self::CanonicalObjectModel => write!(f, "canonicalObjectModel"),
            Self::Validation => write!(f, "validation"),
            Self::Planning => write!(f, "planning"),
            Self::CapabilityEvaluation => write!(f, "capabilityEvaluation"),
            Self::OrchestratorBinding => write!(f, "orchestratorBinding"),
        }
    }
}
