//! Failure semantics validation phase.

use crate::diagnostics::ValidationReport;
use crate::model::PipelineContract;

/// Validate failure-semantics constraints beyond identity.
///
/// Identifier presence and uniqueness are owned by the Canonical Object Model
/// phase. Response and recovery semantics are deferred to ROADMAP 0.6.0.
pub fn validate(_contract: &PipelineContract) -> ValidationReport {
    ValidationReport::new()
}
