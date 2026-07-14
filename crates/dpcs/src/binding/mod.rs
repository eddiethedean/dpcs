//! Orchestrator binding framework (SPEC Ch 17).
//!
//! Translates a validated [`PipelinePlan`](crate::plan::PipelinePlan) into
//! platform-specific scaffold artifacts after a successful capability match.
//!
//! Supported targets: Airflow, Dagster, Prefect, Temporal (experimental), and
//! Kubernetes (experimental). Artifact formats are implementation-defined
//! scaffolds that preserve pipeline identity, topology, and declared intents.

mod adapters;
mod artifact;
mod diagnostics;
mod framework;

pub use artifact::{BindingBundle, BindingFile, BindingTarget};
pub use framework::{
    bind, bind_contract, bind_contract_with_resolve, parse_target, write_bundle, BindContext,
    BindingResult,
};

/// Entry point describing binding availability for this release.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindingFramework;

impl BindingFramework {
    /// Binding is implemented in this release (ROADMAP 0.8.0).
    pub fn is_available() -> bool {
        true
    }

    /// Supported binding targets in stable order.
    pub fn supported_targets() -> &'static [BindingTarget] {
        BindingTarget::all()
    }
}
