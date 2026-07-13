//! # dpcs
//!
//! Reference implementation of the [Data Pipeline Contract Standard](../SPEC.md) (DPCS).
//!
//! Initial processing pipeline:
//!
//! ```text
//! DPCS Document -> Parser -> Canonical Object Model -> Validator -> Diagnostics
//! ```
//!
//! [`SPEC.md`](../SPEC.md) is the authoritative source of truth. When implementation
//! details are ambiguous, this crate prefers the smallest conservative behavior.
//!
//! # Example
//!
//! ```rust,no_run
//! use dpcs::{parse_yaml_file, validate};
//!
//! let contract = parse_yaml_file("pipeline.dpcs.yaml")?;
//! let report = validate(&contract);
//! assert!(report.is_valid());
//! # Ok::<(), dpcs::Error>(())
//! ```

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod binding;
pub mod capabilities;
pub mod diagnostics;
pub mod error;
pub mod model;
pub mod parser;
pub mod plan;
pub mod validation;

#[cfg(feature = "cli")]
pub mod cli;

pub use diagnostics::{Diagnostic, DiagnosticStage, Severity, ValidationReport};
pub use error::{Error, Result};
pub use model::{
    step_id_from_endpoint, unreachable_datasets, unsatisfied_ports, CycleError, DependencyGraph,
    DuplicateEdge, EndpointRole, ExtensionMap, ExtensionValue, IdentityCatalog, InterfacePort,
    Metadata, ObjectId, ObjectKind, ObjectPath, PipelineContract, PipelineGraph, PipelineIdentity,
    PipelineInterface, PipelineStep,
};
pub use parser::{
    parse_file, parse_json, parse_json_file, parse_yaml, parse_yaml_file, to_file, to_json,
    to_json_file, to_yaml, to_yaml_file,
};
pub use validation::validate;

/// Library and CLI version string.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Supported DPCS specification version for this release.
pub const DPCS_SPEC_VERSION: &str = "1.0.0-draft";
