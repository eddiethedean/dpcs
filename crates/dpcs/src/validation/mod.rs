//! Phase-based validation for Pipeline Contracts.

mod cache;
mod com;
mod control_flow;
mod data_flow;
mod document;
mod execution;
mod extensions;
mod failure;
mod governance;
mod graph;
mod lineage;
mod phases;
mod quality;
mod references;
mod scheduling;
mod security;
mod structural;

pub use cache::{validate_cached, ValidationCache, ValidationCacheStats};
pub use extensions::{
    validate_extension_definition, validate_with_options as validate_extensions_with_options,
    ExtensionValidationOptions,
};
pub use governance::validate_governance;
pub use phases::{validate, validate_sequential, validate_with_context, ValidationPhase};
pub use security::validate_security;
