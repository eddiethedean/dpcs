//! Orchestrator capability model (SPEC Ch 16).
//!
//! Capability profiles declare what an orchestrator can support. Matching
//! compares Pipeline Plan execution requirements against a profile without
//! modifying plan semantics.

mod evaluate;
mod profile;

pub use evaluate::{
    evaluate, evaluate_many, evaluate_requirements, CapabilityReport, CapabilityResult,
};
pub use profile::{validate_profile, CapabilityDecl, CapabilityProfile, OrchestratorCapabilities};
