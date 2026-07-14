//! Incremental validation cache keyed by contract section fingerprints.

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use crate::diagnostics::{Diagnostic, ValidationReport};
use crate::model::{AnalysisContext, PipelineContract};

use super::phases::{self, ValidationPhase};

/// Statistics describing the last [`super::validate_cached`] call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationCacheStats {
    /// Phases executed for the last call.
    pub phases_run: usize,
    /// Phases reused from the cache.
    pub phases_reused: usize,
}

/// Reusable validation cache for repeated validation of evolving contracts.
///
/// Fingerprints contract sections aligned to validation phases. Dirty phases are
/// re-run; clean phases reuse prior diagnostics. A cold cache is equivalent to
/// a full [`crate::validate`] pass.
#[derive(Debug, Clone, Default)]
pub struct ValidationCache {
    fingerprints: BTreeMap<ValidationPhase, u64>,
    diagnostics: BTreeMap<ValidationPhase, Vec<Diagnostic>>,
    /// Graph-dependent analysis reuse key (steps + graph + control + data flow).
    analysis_fingerprint: Option<u64>,
    stats: ValidationCacheStats,
}

impl ValidationCache {
    /// Creates an empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears all cached fingerprints and diagnostics.
    pub fn clear(&mut self) {
        self.fingerprints.clear();
        self.diagnostics.clear();
        self.analysis_fingerprint = None;
        self.stats = ValidationCacheStats::default();
    }

    /// Returns statistics from the most recent cached validation.
    pub fn stats(&self) -> &ValidationCacheStats {
        &self.stats
    }
}

/// Validate `contract`, reusing cached phase results when section fingerprints match.
pub fn validate_cached(
    contract: &PipelineContract,
    cache: &mut ValidationCache,
) -> ValidationReport {
    let analysis_fp = fingerprint_analysis_inputs(contract);
    let rebuild_analysis = cache.analysis_fingerprint != Some(analysis_fp);
    let ctx = AnalysisContext::build(contract);
    if rebuild_analysis {
        cache.analysis_fingerprint = Some(analysis_fp);
        // Shared analysis inputs changed: force graph-dependent phases dirty.
        for phase in [
            ValidationPhase::Graph,
            ValidationPhase::ControlFlow,
            ValidationPhase::DataFlow,
            ValidationPhase::References,
            ValidationPhase::Quality,
            ValidationPhase::Failure,
            ValidationPhase::Lineage,
        ] {
            cache.fingerprints.remove(&phase);
        }
    }

    let mut stats = ValidationCacheStats::default();
    let mut report = ValidationReport::new();

    for phase in ValidationPhase::ALL {
        let fp = fingerprint_phase(contract, phase);
        if cache.fingerprints.get(&phase) == Some(&fp) {
            if let Some(cached) = cache.diagnostics.get(&phase) {
                for diagnostic in cached {
                    report.push(diagnostic.clone());
                }
                stats.phases_reused += 1;
                continue;
            }
        }

        let phase_report = phases::run_phase(phase, &ctx);
        cache.fingerprints.insert(phase, fp);
        cache
            .diagnostics
            .insert(phase, phase_report.diagnostics.clone());
        report.extend(phase_report);
        stats.phases_run += 1;
    }

    cache.stats = stats;
    report.sort_deterministic();
    report
}

fn fingerprint_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

fn fingerprint_json<T: serde::Serialize>(value: &T) -> u64 {
    match serde_json::to_vec(value) {
        Ok(bytes) => fingerprint_bytes(&bytes),
        Err(_) => 0,
    }
}

fn fingerprint_analysis_inputs(contract: &PipelineContract) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    fingerprint_json(&contract.steps).hash(&mut hasher);
    fingerprint_json(&contract.graph).hash(&mut hasher);
    fingerprint_json(&contract.control_flow).hash(&mut hasher);
    fingerprint_json(&contract.data_flow).hash(&mut hasher);
    hasher.finish()
}

fn fingerprint_phase(contract: &PipelineContract, phase: ValidationPhase) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    match phase {
        ValidationPhase::Document => {
            contract.dpcs_version.hash(&mut hasher);
        }
        ValidationPhase::Com | ValidationPhase::Structural => {
            fingerprint_json(&contract.id).hash(&mut hasher);
            fingerprint_json(&contract.version).hash(&mut hasher);
            fingerprint_json(&contract.dpcs_version).hash(&mut hasher);
            fingerprint_json(&contract.name).hash(&mut hasher);
            fingerprint_json(&contract.interface).hash(&mut hasher);
            fingerprint_json(&contract.steps).hash(&mut hasher);
            fingerprint_json(&contract.contract_references).hash(&mut hasher);
            fingerprint_json(&contract.quality_gates).hash(&mut hasher);
            fingerprint_json(&contract.failure_semantics).hash(&mut hasher);
            fingerprint_json(&contract.scheduling).hash(&mut hasher);
            fingerprint_json(&contract.extensions).hash(&mut hasher);
        }
        ValidationPhase::Graph
        | ValidationPhase::ControlFlow
        | ValidationPhase::DataFlow
        | ValidationPhase::References
        | ValidationPhase::Quality
        | ValidationPhase::Failure
        | ValidationPhase::Lineage => {
            fingerprint_analysis_inputs(contract).hash(&mut hasher);
            fingerprint_json(&contract.contract_references).hash(&mut hasher);
            fingerprint_json(&contract.quality_gates).hash(&mut hasher);
            fingerprint_json(&contract.failure_semantics).hash(&mut hasher);
            fingerprint_json(&contract.lineage).hash(&mut hasher);
            fingerprint_json(&contract.interface).hash(&mut hasher);
        }
        ValidationPhase::Execution => {
            fingerprint_json(&contract.execution).hash(&mut hasher);
        }
        ValidationPhase::Scheduling => {
            fingerprint_json(&contract.scheduling).hash(&mut hasher);
        }
        ValidationPhase::Security => {
            fingerprint_json(&contract.security).hash(&mut hasher);
        }
        ValidationPhase::Governance => {
            fingerprint_json(&contract.governance).hash(&mut hasher);
        }
        ValidationPhase::Extensions => {
            fingerprint_json(&contract.extensions).hash(&mut hasher);
            fingerprint_json(&contract.compatibility).hash(&mut hasher);
        }
    }
    hasher.finish()
}
