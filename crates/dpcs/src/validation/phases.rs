//! Validation phase orchestration.

use crate::diagnostics::ValidationReport;
use crate::model::{AnalysisContext, PipelineContract};

use super::{
    com, control_flow, data_flow, document, execution, extensions, failure, governance, graph,
    lineage, quality, references, scheduling, security, structural,
};

/// Named validation phase used by orchestration and incremental caching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValidationPhase {
    /// Document-level checks.
    Document,
    /// Canonical Object Model invariants.
    Com,
    /// Structural checks.
    Structural,
    /// Graph topology.
    Graph,
    /// Reference resolution.
    References,
    /// Data-flow wiring.
    DataFlow,
    /// Control-flow conflicts.
    ControlFlow,
    /// Execution requirements.
    Execution,
    /// Scheduling intents.
    Scheduling,
    /// Quality gates.
    Quality,
    /// Failure semantics.
    Failure,
    /// Lineage.
    Lineage,
    /// Security metadata.
    Security,
    /// Governance metadata.
    Governance,
    /// Extensions / compatibility.
    Extensions,
}

impl ValidationPhase {
    /// Phases in the SPEC-aligned orchestration order.
    pub const ALL: [Self; 15] = [
        Self::Document,
        Self::Com,
        Self::Structural,
        Self::Graph,
        Self::References,
        Self::DataFlow,
        Self::ControlFlow,
        Self::Execution,
        Self::Scheduling,
        Self::Quality,
        Self::Failure,
        Self::Lineage,
        Self::Security,
        Self::Governance,
        Self::Extensions,
    ];
}

/// Validate a Pipeline Contract using the DPCS phase model.
///
/// When the `parallel` feature is enabled, independent phases run concurrently
/// and diagnostics are merged then sorted deterministically. Without that
/// feature this is identical to [`validate_sequential`].
pub fn validate(contract: &PipelineContract) -> ValidationReport {
    let ctx = AnalysisContext::build(contract);
    validate_with_context(&ctx)
}

/// Validate using a prebuilt [`AnalysisContext`].
pub fn validate_with_context(ctx: &AnalysisContext<'_>) -> ValidationReport {
    #[cfg(feature = "parallel")]
    {
        validate_parallel(ctx)
    }
    #[cfg(not(feature = "parallel"))]
    {
        validate_sequential_with_context(ctx)
    }
}

/// Always-sequential validation (useful for benches and determinism checks).
pub fn validate_sequential(contract: &PipelineContract) -> ValidationReport {
    let ctx = AnalysisContext::build(contract);
    validate_sequential_with_context(&ctx)
}

/// Sequential validation using a prebuilt context.
pub fn validate_sequential_with_context(ctx: &AnalysisContext<'_>) -> ValidationReport {
    let mut report = ValidationReport::new();
    for phase in ValidationPhase::ALL {
        report.extend(run_phase(phase, ctx));
    }
    report.sort_deterministic();
    report
}

#[cfg(feature = "parallel")]
fn validate_parallel(ctx: &AnalysisContext<'_>) -> ValidationReport {
    use rayon::prelude::*;

    let mut reports: Vec<ValidationReport> = ValidationPhase::ALL
        .into_par_iter()
        .map(|phase| run_phase(phase, ctx))
        .collect();

    let mut report = ValidationReport::new();
    for phase_report in reports.drain(..) {
        report.extend(phase_report);
    }
    report.sort_deterministic();
    report
}

/// Run a single validation phase against a shared analysis context.
pub(crate) fn run_phase(phase: ValidationPhase, ctx: &AnalysisContext<'_>) -> ValidationReport {
    let contract = ctx.contract;
    match phase {
        ValidationPhase::Document => document::validate(contract),
        ValidationPhase::Com => com::validate(contract),
        ValidationPhase::Structural => structural::validate(contract),
        ValidationPhase::Graph => graph::validate_with_context(ctx),
        ValidationPhase::References => references::validate_with_context(ctx),
        ValidationPhase::DataFlow => data_flow::validate_with_context(ctx),
        ValidationPhase::ControlFlow => control_flow::validate_with_context(ctx),
        ValidationPhase::Execution => execution::validate(contract),
        ValidationPhase::Scheduling => scheduling::validate(contract),
        ValidationPhase::Quality => quality::validate_with_context(ctx),
        ValidationPhase::Failure => failure::validate_with_context(ctx),
        ValidationPhase::Lineage => lineage::validate_with_context(ctx),
        ValidationPhase::Security => security::validate(contract),
        ValidationPhase::Governance => governance::validate(contract),
        ValidationPhase::Extensions => extensions::validate(contract),
    }
}
