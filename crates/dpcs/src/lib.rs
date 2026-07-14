//! # dpcs
//!
//! Reference implementation of the [Data Pipeline Contract Standard](../../SPEC.md) (DPCS).
//!
//! Processing pipeline:
//!
//! ```text
//! DPCS Document -> Parser -> COM -> Validator -> Plan -> Capabilities -> Binding
//! Compatibility / Registry / Conformance are first-class analysis surfaces.
//! ```
//!
//! [`SPEC.md`](../../SPEC.md) is the authoritative source of truth. When implementation
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
pub mod compatibility;
pub mod conformance;
pub mod diagnostics;
pub mod error;
pub mod model;
pub mod package;
pub mod parser;
pub mod plan;
pub mod validation;

mod paths;

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(any(feature = "registry-client", feature = "registry-server"))]
pub mod registry_net;
#[cfg(feature = "jsonschema")]
pub mod schema;

pub use binding::{
    bind, bind_contract, parse_target, write_bundle, BindContext, BindingBundle, BindingFile,
    BindingFramework, BindingResult, BindingTarget,
};
#[allow(deprecated)]
pub use capabilities::OrchestratorCapabilities;
pub use capabilities::{
    evaluate, evaluate_many, evaluate_requirements, validate_profile, CapabilityDecl,
    CapabilityProfile, CapabilityReport, CapabilityResult,
};
pub use compatibility::{
    compare_contracts, compare_plans, CompatibilityCategory, CompatibilityReport,
    CompatibilityResult,
};
pub use conformance::{
    apply_profile_to_contract, implemented_levels, toolkit_claim, validate_claim,
    validate_profile as validate_conformance_profile, ConformanceClaim, ConformanceLevel,
    ConformanceProfile,
};
pub use diagnostics::{
    validate_diagnostic, Diagnostic, DiagnosticReport, DiagnosticStage, ImplementationMetadata,
    ProcessingResult, Severity, ValidationReport,
};
pub use error::{Error, Result};
pub use model::{
    is_valid_extension_namespace, is_valid_version, parse_version, step_id_from_endpoint,
    unreachable_datasets, unsatisfied_ports, validate_registry, versions_compatible,
    CompatibilityPolicy, ContractReference, CycleError, DatasetLineage, DependencyGraph,
    DuplicateEdge, EndpointRole, ExecutionEnvironment, ExecutionRequirements, ExtensionDefinition,
    ExtensionMap, ExtensionValue, ExternalDependency, FailureResponse, FailureScope,
    FailureSemantics, GateOutcome, GatePlacement, GovernanceMetadata, IdentityCatalog,
    IntegrityReference, InterfacePort, Metadata, ObjectId, ObjectKind, ObjectPath, ParsedVersion,
    PipelineContract, PipelineGraph, PipelineIdentity, PipelineInterface, PipelineLineage,
    PipelineProvenance, PipelineStep, QualityCriterion, QualityGate, RegisteredArtifact, Registry,
    RegistryReference, ResourceRequirements, RetrySemantics, SchedulingConstraints,
    SchedulingEvent, SchedulingIntent, SchedulingMode, SecretReference, SecurityMetadata,
    StepLineage,
};
pub use package::{
    artifact_entry, pack, resolve_artifact, unpack, validate_package, write_manifest,
    PackageArtifactEntry, PackageLayout, PackageManifest,
};
pub use parser::{
    parse_file, parse_json, parse_json_file, parse_yaml, parse_yaml_file, to_file, to_json,
    to_json_file, to_yaml, to_yaml_file,
};
pub use plan::{plan, try_plan, PipelinePlan, PlanDependencyEdge, PlanResult};
pub use validation::{
    validate, validate_extension_definition, validate_governance, validate_security,
};

#[cfg(feature = "jsonschema")]
pub use schema::{
    document_schemas, json_schema_for, openapi_document, schema_to_value, write_document_schemas,
    write_openapi_documents, OpenApiKind,
};

#[cfg(any(feature = "registry-client", feature = "registry-server"))]
pub use registry_net::PublishRequest;
#[cfg(feature = "registry-client")]
pub use registry_net::{RegistryCache, RegistryClient, RegistryClientError};

#[cfg(feature = "registry-server")]
pub use registry_net::{serve, serve_listener, ServeOptions};

/// Library and CLI version string.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Supported DPCS specification version for this release.
pub const DPCS_SPEC_VERSION: &str = "1.0.0-draft";
