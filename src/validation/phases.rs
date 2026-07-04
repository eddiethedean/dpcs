//! Validation phase orchestration.

use crate::diagnostics::ValidationReport;
use crate::model::PipelineContract;

use super::{
    control_flow, data_flow, document, extensions, failure, graph, quality, references, structural,
};

/// Validate a Pipeline Contract using the DPCS phase model.
///
/// Phases run in order and always complete. Findings are accumulated into a
/// single deterministic [`ValidationReport`].
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let mut report = ValidationReport::new();

    report.extend(document::validate(contract));
    report.extend(structural::validate(contract));
    report.extend(graph::validate(contract));
    report.extend(references::validate(contract));
    report.extend(data_flow::validate(contract));
    report.extend(control_flow::validate(contract));
    report.extend(quality::validate(contract));
    report.extend(failure::validate(contract));
    report.extend(extensions::validate(contract));

    report.sort_deterministic();
    report
}
