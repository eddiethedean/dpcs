//! Capability matching against Pipeline Plans (SPEC Ch 16).

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{versions_compatible, ExecutionRequirements};
use crate::plan::PipelinePlan;
use crate::DPCS_SPEC_VERSION;

use super::profile::{validate_profile, CapabilityDecl, CapabilityProfile};

/// Structured capability evaluation report (ROADMAP “Capability reports”).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CapabilityReport {
    /// Identity of the evaluated profile.
    pub profile_identity: String,
    /// Contract id from the plan when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_contract_id: Option<String>,
    /// Capability ids demanded by the plan and satisfied by the profile.
    pub satisfied: Vec<String>,
    /// Mandatory capability ids demanded by the plan but missing from the profile.
    pub missing_mandatory: Vec<String>,
    /// Profile capabilities marked `optional: true` that were not demanded.
    ///
    /// Informational only: these are unused optional supply, not SPEC “unsupported
    /// optional capabilities” that may be ignored on the demand side.
    pub unsupported_optional: Vec<String>,
    /// Non-error diagnostics observed during evaluation (for example version warnings).
    ///
    /// On CLI JSON failure paths, this may be replaced with the full diagnostic list.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<Diagnostic>,
}

/// Result of evaluating a plan (or requirements) against a capability profile.
#[derive(Debug, Clone, PartialEq)]
pub enum CapabilityResult {
    /// Evaluation succeeded; report includes match details.
    Ok(Box<CapabilityReport>),
    /// Evaluation failed; structured match report plus CapabilityEvaluation diagnostics.
    Err {
        /// Partial or complete match report (includes `missing_mandatory` when applicable).
        report: Box<CapabilityReport>,
        /// Diagnostics that caused the failure (and any accompanying warnings).
        diagnostics: ValidationReport,
    },
}

impl CapabilityResult {
    /// Returns the report when evaluation succeeded.
    pub fn report(self) -> Option<CapabilityReport> {
        match self {
            Self::Ok(report) => Some(*report),
            Self::Err { .. } => None,
        }
    }

    /// Returns a reference to the structured capability report (success or failure).
    pub fn as_capability_report(&self) -> &CapabilityReport {
        match self {
            Self::Ok(report) => report,
            Self::Err { report, .. } => report,
        }
    }

    /// Returns a reference to the report when evaluation succeeded.
    pub fn as_report(&self) -> Option<&CapabilityReport> {
        match self {
            Self::Ok(report) => Some(report),
            Self::Err { .. } => None,
        }
    }

    /// Returns whether evaluation succeeded.
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    /// Returns diagnostics when evaluation failed.
    pub fn diagnostics(&self) -> Option<&ValidationReport> {
        match self {
            Self::Ok(_) => None,
            Self::Err { diagnostics, .. } => Some(diagnostics),
        }
    }
}

/// Evaluate a Pipeline Plan against an orchestrator capability profile.
pub fn evaluate(plan: &PipelinePlan, profile: &CapabilityProfile) -> CapabilityResult {
    match_requirements(
        plan.execution.as_ref(),
        profile,
        Some(plan.contract_id.clone()),
        Some(plan.dpcs_version.as_str()),
    )
}

/// Evaluate declared execution requirements against a capability profile.
///
/// Supports Ch 10 §8 checks when a target profile is known before planning.
pub fn evaluate_requirements(
    requirements: &ExecutionRequirements,
    profile: &CapabilityProfile,
) -> CapabilityResult {
    match_requirements(Some(requirements), profile, None, None)
}

/// Evaluate a plan against multiple profiles and rank best matches first.
///
/// Ranking prefers successful matches, then fewer missing mandatories, then more
/// satisfied capabilities, then profile identity (deterministic).
pub fn evaluate_many(
    plan: &PipelinePlan,
    profiles: &[CapabilityProfile],
) -> Vec<(String, CapabilityResult)> {
    let mut results: Vec<(String, CapabilityResult)> = profiles
        .iter()
        .map(|profile| (profile.identity.clone(), evaluate(plan, profile)))
        .collect();

    results.sort_by(|(id_a, result_a), (id_b, result_b)| {
        rank_key(result_a)
            .cmp(&rank_key(result_b))
            .then_with(|| id_a.cmp(id_b))
    });
    results
}

fn rank_key(result: &CapabilityResult) -> (u8, usize, Reverse<usize>) {
    let report = result.as_capability_report();
    let tier = match result {
        CapabilityResult::Ok(_) => 0u8,
        CapabilityResult::Err {
            diagnostics,
            report,
        } => {
            let profile_invalid = diagnostics.diagnostics.iter().any(|diagnostic| {
                matches!(
                    diagnostic.id.as_str(),
                    "DPCS-CAP-001" | "DPCS-CAP-002" | "DPCS-CAP-003" | "DPCS-CAP-004"
                )
            });
            // Invalid profiles (no successful demand match) rank after incomplete matches.
            if profile_invalid && report.missing_mandatory.is_empty() && report.satisfied.is_empty()
            {
                2
            } else {
                1
            }
        }
    };
    (
        tier,
        report.missing_mandatory.len(),
        Reverse(report.satisfied.len()),
    )
}

fn match_requirements(
    requirements: Option<&ExecutionRequirements>,
    profile: &CapabilityProfile,
    plan_contract_id: Option<String>,
    plan_dpcs_version: Option<&str>,
) -> CapabilityResult {
    let mut report = validate_profile(profile);
    if !report.is_valid() {
        report.sort_deterministic();
        return CapabilityResult::Err {
            report: Box::new(empty_report(profile, plan_contract_id)),
            diagnostics: report,
        };
    }

    maybe_version_warning(profile, plan_dpcs_version, &mut report);

    let demand = demand_entries(requirements);
    let mut by_id: BTreeMap<&str, &CapabilityDecl> = BTreeMap::new();
    for capability in &profile.capabilities {
        let id = capability.id.trim();
        if !id.is_empty() {
            by_id.entry(id).or_insert(capability);
        }
    }

    let mut satisfied = Vec::new();
    let mut missing_mandatory = Vec::new();

    for (demanded, source) in &demand {
        if by_id.contains_key(demanded.as_str()) {
            satisfied.push(demanded.clone());
        } else {
            missing_mandatory.push(demanded.clone());
            report.push(
                Diagnostic::capability_error(
                    "DPCS-CAP-005",
                    categories::CAPABILITY,
                    format!("unsupported mandatory capability `{demanded}`"),
                )
                .with_object_ref(source.object_ref(demanded))
                .with_remediation(format!(
                    "Add `{demanded}` to the orchestrator capability profile or remove it from execution requirements"
                )),
            );
        }
    }

    let demand_ids: BTreeSet<&str> = demand.keys().map(String::as_str).collect();
    let unsupported_optional: Vec<String> = profile
        .capabilities
        .iter()
        .filter(|capability| capability.optional)
        .map(|capability| capability.id.trim().to_owned())
        .filter(|id| !id.is_empty() && !demand_ids.contains(id.as_str()))
        .collect();

    report.sort_deterministic();

    let warnings: Vec<Diagnostic> = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == crate::diagnostics::Severity::Warning)
        .cloned()
        .collect();

    let capability_report = CapabilityReport {
        profile_identity: profile.identity.clone(),
        plan_contract_id,
        satisfied,
        missing_mandatory,
        unsupported_optional,
        diagnostics: warnings,
    };

    if report.error_count() > 0 {
        CapabilityResult::Err {
            report: Box::new(capability_report),
            diagnostics: report,
        }
    } else {
        CapabilityResult::Ok(Box::new(capability_report))
    }
}

fn empty_report(profile: &CapabilityProfile, plan_contract_id: Option<String>) -> CapabilityReport {
    CapabilityReport {
        profile_identity: profile.identity.clone(),
        plan_contract_id,
        satisfied: Vec::new(),
        missing_mandatory: Vec::new(),
        unsupported_optional: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn maybe_version_warning(
    profile: &CapabilityProfile,
    plan_dpcs_version: Option<&str>,
    report: &mut ValidationReport,
) {
    let profile_version = profile.dpcs_version.trim();
    if profile_version.is_empty() {
        return;
    }

    let toolkit_mismatch = !versions_compatible(profile_version, DPCS_SPEC_VERSION);
    let plan_version = plan_dpcs_version
        .map(str::trim)
        .filter(|version| !version.is_empty());
    let plan_mismatch = plan_version
        .is_some_and(|plan_version| !versions_compatible(profile_version, plan_version));

    if !toolkit_mismatch && !plan_mismatch {
        return;
    }

    let message = match plan_version {
        Some(plan_version) if plan_mismatch => format!(
            "profile dpcsVersion `{profile_version}` differs from plan/contract dpcsVersion `{plan_version}` (toolkit `{DPCS_SPEC_VERSION}`)"
        ),
        _ => format!(
            "profile dpcsVersion `{profile_version}` differs from supported toolkit version `{DPCS_SPEC_VERSION}`"
        ),
    };

    report.push(
        Diagnostic::capability_warning("DPCS-CAP-006", categories::CAPABILITY, message)
            .with_object_ref("dpcsVersion")
            .with_remediation(
                "Align profile dpcsVersion with the contract and toolkit DPCS version when possible",
            ),
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DemandSource {
    RequiredCapability,
    ExternalDependency,
}

impl DemandSource {
    fn object_ref(self, capability: &str) -> String {
        match self {
            Self::RequiredCapability => {
                format!("execution.requiredCapabilities:{capability}")
            }
            Self::ExternalDependency => {
                format!("execution.externalDependencies.capability:{capability}")
            }
        }
    }
}

/// Build the mandatory demand set for orchestrator matching (SPEC Ch 16).
///
/// Demand includes:
/// - `execution.requiredCapabilities`
/// - `execution.externalDependencies[].capability`
///
/// Environment `softwareCapabilities` and `isolation` are contract-environment
/// characteristics, not orchestrator capability ids, and are not matched here.
fn demand_entries(requirements: Option<&ExecutionRequirements>) -> BTreeMap<String, DemandSource> {
    let mut demand = BTreeMap::new();
    let Some(execution) = requirements else {
        return demand;
    };

    for capability in &execution.required_capabilities {
        let trimmed = capability.trim();
        if !trimmed.is_empty() {
            demand
                .entry(trimmed.to_owned())
                .or_insert(DemandSource::RequiredCapability);
        }
    }

    for dependency in &execution.external_dependencies {
        let trimmed = dependency.capability.trim();
        if !trimmed.is_empty() {
            demand
                .entry(trimmed.to_owned())
                .or_insert(DemandSource::ExternalDependency);
        }
    }

    demand
}
