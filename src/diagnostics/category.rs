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
}
