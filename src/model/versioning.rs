//! Versioning helpers for DPCS artifacts.

/// Returns `true` when `version` looks like a non-empty version string.
///
/// Full semver enforcement is deferred to roadmap 0.9.0.
pub fn is_present_version(version: &str) -> bool {
    !version.trim().is_empty()
}
