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
}
