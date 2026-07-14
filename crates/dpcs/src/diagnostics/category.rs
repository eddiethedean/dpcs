//! Diagnostic categories.

/// Common diagnostic category identifiers.
pub mod categories {
    /// Document-level issues.
    pub const DOCUMENT: &str = "document";
    /// Canonical Object Model issues.
    pub const CANONICAL_OBJECT_MODEL: &str = "canonicalObjectModel";
    /// Structural issues.
    pub const STRUCTURAL: &str = "structural";
    /// Graph issues.
    pub const GRAPH: &str = "graph";
    /// Contract reference issues.
    pub const REFERENCE: &str = "reference";
    /// Data-flow issues.
    pub const DATA_FLOW: &str = "dataFlow";
    /// Control-flow issues.
    pub const CONTROL_FLOW: &str = "controlFlow";
    /// Extension issues.
    pub const EXTENSION: &str = "extension";
    /// Syntax and parse issues.
    pub const SYNTAX: &str = "syntax";
    /// Execution requirements issues.
    pub const EXECUTION_REQUIREMENTS: &str = "executionRequirements";
    /// Scheduling intent issues.
    pub const SCHEDULING: &str = "scheduling";
    /// Quality gate issues.
    pub const QUALITY_GATES: &str = "qualityGates";
    /// Failure semantics issues.
    pub const FAILURE_SEMANTICS: &str = "failureSemantics";
    /// Lineage issues.
    pub const LINEAGE: &str = "lineage";
    /// Planning issues.
    pub const PLANNING: &str = "planning";
    /// Capability evaluation issues.
    pub const CAPABILITY: &str = "capability";
    /// Orchestrator binding issues.
    pub const BINDING: &str = "binding";
    /// Compatibility analysis issues.
    pub const COMPATIBILITY: &str = "compatibility";
    /// Versioning issues.
    pub const VERSIONING: &str = "versioning";
    /// Registry issues.
    pub const REGISTRY: &str = "registry";
    /// Conformance claim / profile issues.
    pub const CONFORMANCE: &str = "conformance";
    /// Security metadata issues.
    pub const SECURITY: &str = "security";
    /// Governance metadata issues.
    pub const GOVERNANCE: &str = "governance";
    /// Pipeline package issues.
    pub const PACKAGE: &str = "package";
    /// Registry client / network protocol issues.
    pub const REGISTRY_CLIENT: &str = "registryClient";
}
