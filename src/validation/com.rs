//! Canonical Object Model validation phase.

use crate::diagnostics::ValidationReport;
use crate::model::{validate_com_invariants, PipelineContract};

/// Validate Canonical Object Model invariants.
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    validate_com_invariants(contract)
}
