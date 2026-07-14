//! Capability matching against Pipeline Plans (SPEC Ch 16).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::ExecutionRequirements;
use crate::plan::PipelinePlan;
use crate::DPCS_SPEC_VERSION;

use super::profile::{validate_profile, CapabilityDecl, CapabilityProfile};

/// Structured capability evaluation report (ROADMAP “Capability reports”).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Optional profile capabilities that were not demanded by the plan.
    pub unsupported_optional: Vec<String>,
    /// Non-error diagnostics observed during evaluation (for example version warnings).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<Diagnostic>,
}

/// Result of evaluating a plan (or requirements) against a capability profile.
#[derive(Debug, Clone, PartialEq)]
pub enum CapabilityResult {
    /// Evaluation succeeded; report includes match details.
    Ok(Box<CapabilityReport>),
    /// Evaluation failed with CapabilityEvaluation-stage diagnostics.
    Err(ValidationReport),
}

impl CapabilityResult {
    /// Returns the report when evaluation succeeded.
    pub fn report(self) -> Option<CapabilityReport> {
        match self {
            Self::Ok(report) => Some(*report),
            Self::Err(_) => None,
        }
    }

    /// Returns a reference to the report when evaluation succeeded.
    pub fn as_report(&self) -> Option<&CapabilityReport> {
        match self {
            Self::Ok(report) => Some(report),
            Self::Err(_) => None,
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
            Self::Err(report) => Some(report),
        }
    }
}

/// Evaluate a Pipeline Plan against an orchestrator capability profile.
pub fn evaluate(plan: &PipelinePlan, profile: &CapabilityProfile) -> CapabilityResult {
    match_requirements(
        plan.execution.as_ref(),
        profile,
        Some(plan.contract_id.clone()),
    )
}

/// Evaluate declared execution requirements against a capability profile.
///
/// Supports Ch 10 §8 checks when a target profile is known before planning.
pub fn evaluate_requirements(
    requirements: &ExecutionRequirements,
    profile: &CapabilityProfile,
) -> CapabilityResult {
    match_requirements(Some(requirements), profile, None)
}

/// Evaluate a plan against multiple profiles (minimal Ch 16 §7 selection helper).
pub fn evaluate_many(
    plan: &PipelinePlan,
    profiles: &[CapabilityProfile],
) -> Vec<(String, CapabilityResult)> {
    profiles
        .iter()
        .map(|profile| (profile.identity.clone(), evaluate(plan, profile)))
        .collect()
}

fn match_requirements(
    requirements: Option<&ExecutionRequirements>,
    profile: &CapabilityProfile,
    plan_contract_id: Option<String>,
) -> CapabilityResult {
    let mut report = validate_profile(profile);
    if !report.is_valid() {
        return CapabilityResult::Err(report);
    }

    if profile_version_incompatible(&profile.dpcs_version) {
        report.push(
            Diagnostic::capability_warning(
                "DPCS-CAP-006",
                categories::CAPABILITY,
                format!(
                    "profile dpcsVersion `{}` differs from supported toolkit version `{DPCS_SPEC_VERSION}`",
                    profile.dpcs_version.trim()
                ),
            )
            .with_object_ref("dpcsVersion")
            .with_remediation(
                "Align profile dpcsVersion with the contract/toolkit DPCS version when possible",
            ),
        );
    }

    let demand = demand_set(requirements);
    let mut by_id: BTreeMap<&str, &CapabilityDecl> = BTreeMap::new();
    for capability in &profile.capabilities {
        let id = capability.id.trim();
        if !id.is_empty() {
            by_id.entry(id).or_insert(capability);
        }
    }

    let mut satisfied = Vec::new();
    let mut missing_mandatory = Vec::new();

    for demanded in &demand {
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
                .with_object_ref(format!("execution.requiredCapabilities:{demanded}"))
                .with_remediation(format!(
                    "Add `{demanded}` to the orchestrator capability profile or remove it from execution requirements"
                )),
            );
        }
    }

    let unsupported_optional: Vec<String> = profile
        .capabilities
        .iter()
        .filter(|capability| capability.optional)
        .map(|capability| capability.id.trim().to_owned())
        .filter(|id| !id.is_empty() && !demand.contains(id))
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
        CapabilityResult::Err(report)
    } else {
        CapabilityResult::Ok(Box::new(capability_report))
    }
}

fn demand_set(requirements: Option<&ExecutionRequirements>) -> BTreeSet<String> {
    let mut demand = BTreeSet::new();
    let Some(execution) = requirements else {
        return demand;
    };

    for capability in &execution.required_capabilities {
        let trimmed = capability.trim();
        if !trimmed.is_empty() {
            demand.insert(trimmed.to_owned());
        }
    }

    for dependency in &execution.external_dependencies {
        let trimmed = dependency.capability.trim();
        if !trimmed.is_empty() {
            demand.insert(trimmed.to_owned());
        }
    }

    if let Some(environment) = &execution.environment {
        for capability in &environment.software_capabilities {
            let trimmed = capability.trim();
            if !trimmed.is_empty() {
                demand.insert(trimmed.to_owned());
            }
        }
    }

    demand
}

fn profile_version_incompatible(profile_version: &str) -> bool {
    let profile = profile_version.trim();
    if profile.is_empty() {
        return false;
    }
    !versions_compatible(profile, DPCS_SPEC_VERSION)
}

fn versions_compatible(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    let normalize = |value: &str| {
        value
            .trim()
            .trim_end_matches("-draft")
            .split('.')
            .take(2)
            .collect::<Vec<_>>()
            .join(".")
    };
    normalize(left) == normalize(right)
}
