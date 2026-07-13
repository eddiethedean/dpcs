//! Orchestrator binding framework.
//!
//! Binding adapters (Airflow, Dagster, Prefect, and others) are intentionally
//! out of scope until roadmap 0.8.0.

/// Placeholder for future orchestrator binding APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindingFramework;

impl BindingFramework {
    /// Binding is not implemented in this release.
    pub fn is_available() -> bool {
        false
    }
}
