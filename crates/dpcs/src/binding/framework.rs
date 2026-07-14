//! Binding framework: capability gate and adapter dispatch (SPEC Ch 17).

use std::fs;
use std::path::Path;
use std::str::FromStr;

use crate::capabilities::{evaluate, CapabilityProfile, CapabilityReport, CapabilityResult};
use crate::diagnostics::ValidationReport;
use crate::model::PipelineContract;
use crate::paths::join_under_root;
use crate::plan::{self, PipelinePlan, PlanResult};

use super::adapters::{adapter_for, validate_relative_path};
use super::artifact::{BindingBundle, BindingTarget};
use super::diagnostics::{self, write_failed};

/// Context passed to orchestrator adapters during translation.
#[derive(Debug, Clone)]
pub struct BindContext<'a> {
    /// Capability profile identity.
    pub profile_identity: &'a str,
    /// Successful capability evaluation report.
    pub capability: &'a CapabilityReport,
}

/// Result of attempting to bind a Pipeline Plan to an orchestrator target.
#[derive(Debug, Clone, PartialEq)]
pub enum BindingResult {
    /// Binding succeeded and produced a platform artifact bundle.
    Ok(Box<BindingBundle>),
    /// Binding refused (capability gate, planning, or translation failure).
    Err {
        /// Binding-stage (and related) diagnostics.
        diagnostics: ValidationReport,
        /// Structured capability report when refusal is due to capability matching.
        capability: Option<Box<CapabilityReport>>,
    },
}

impl BindingResult {
    /// Returns the bundle when binding succeeded.
    pub fn bundle(self) -> Option<BindingBundle> {
        match self {
            Self::Ok(bundle) => Some(*bundle),
            Self::Err { .. } => None,
        }
    }

    /// Returns a reference to the bundle when binding succeeded.
    pub fn as_bundle(&self) -> Option<&BindingBundle> {
        match self {
            Self::Ok(bundle) => Some(bundle),
            Self::Err { .. } => None,
        }
    }

    /// Returns whether binding succeeded.
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    /// Returns diagnostics when binding failed.
    pub fn report(&self) -> Option<&ValidationReport> {
        match self {
            Self::Ok(_) => None,
            Self::Err { diagnostics, .. } => Some(diagnostics),
        }
    }

    /// Returns the capability report retained on capability-gate refusal.
    pub fn capability_report(&self) -> Option<&CapabilityReport> {
        match self {
            Self::Ok(_) => None,
            Self::Err { capability, .. } => capability.as_deref(),
        }
    }

    fn err(diagnostics: ValidationReport) -> Self {
        Self::Err {
            diagnostics,
            capability: None,
        }
    }

    fn err_with_capability(diagnostics: ValidationReport, report: CapabilityReport) -> Self {
        Self::Err {
            diagnostics,
            capability: Some(Box::new(report)),
        }
    }
}

/// Parse a binding target name into [`BindingTarget`].
///
/// On failure returns a binding-stage diagnostic report with `DPCS-BIND-002`.
pub fn parse_target(name: &str) -> Result<BindingTarget, ValidationReport> {
    BindingTarget::from_str(name)
        .map_err(|_| diagnostics::report_error(diagnostics::unknown_target(name)))
}

/// Bind a validated Pipeline Plan to a target orchestrator.
///
/// Runs capability evaluation against `profile` first. Missing mandatory
/// capabilities refuse binding with `DPCS-BIND-001`. On success, translates the
/// plan into platform-specific scaffold artifacts.
pub fn bind(
    plan: &PipelinePlan,
    profile: &CapabilityProfile,
    target: BindingTarget,
) -> BindingResult {
    let capability = match evaluate(plan, profile) {
        CapabilityResult::Ok(report) => report,
        CapabilityResult::Err {
            report,
            diagnostics,
        } => {
            return BindingResult::err_with_capability(
                diagnostics::report_capability_gate(diagnostics),
                *report,
            );
        }
    };

    let ctx = BindContext {
        profile_identity: &profile.identity,
        capability: &capability,
    };

    let adapter = adapter_for(target);
    match adapter.translate(plan, &ctx) {
        Ok(files) => {
            if files.is_empty() {
                return BindingResult::err(diagnostics::report_error(
                    diagnostics::translation_incomplete("binding adapter produced no artifacts"),
                ));
            }
            for file in &files {
                if let Err(report) = validate_relative_path(&file.relative_path) {
                    return BindingResult::err(report);
                }
            }
            BindingResult::Ok(Box::new(BindingBundle {
                target,
                contract_id: plan.contract_id.clone(),
                contract_version: plan.contract_version.clone(),
                profile_identity: profile.identity.clone(),
                files,
                capability: *capability,
            }))
        }
        Err(report) => BindingResult::err(report),
    }
}

/// Plan a contract, then bind it to a target orchestrator.
///
/// Planning failures are returned as [`BindingResult::Err`] with the planning
/// diagnostics (including `DPCS-PLN-001` when validation failed).
pub fn bind_contract(
    contract: &PipelineContract,
    profile: &CapabilityProfile,
    target: BindingTarget,
) -> BindingResult {
    // Deep-resolve with default planning options (CWD); prefer
    // `bind_contract_with_resolve` + `ResolveOptions::from_document_path` for
    // document-relative nested locations.
    bind_contract_with_resolve(contract, profile, target, None)
}

/// Plan (with reference resolution) then bind.
///
/// When `resolve` is `None`, uses [`crate::ResolveOptions::default_for_planning`].
pub fn bind_contract_with_resolve(
    contract: &PipelineContract,
    profile: &CapabilityProfile,
    target: BindingTarget,
    resolve: Option<&crate::resolve::ResolveOptions>,
) -> BindingResult {
    match plan::plan_with_resolve(contract, resolve) {
        PlanResult::Ok(planned) => bind(&planned, profile, target),
        PlanResult::Err(report) => BindingResult::err(report),
    }
}

/// Write a binding bundle's files under `out_dir`.
///
/// Creates parent directories as needed. Rejects absolute paths, `..` segments,
/// and empty relative paths (`DPCS-BIND-004`). Returns a binding-stage
/// diagnostic report on filesystem errors.
pub fn write_bundle(bundle: &BindingBundle, out_dir: &Path) -> Result<(), ValidationReport> {
    if let Err(err) = fs::create_dir_all(out_dir) {
        return Err(diagnostics::report_error(write_failed(format!(
            "failed to create directory {}: {err}",
            out_dir.display()
        ))));
    }
    for file in &bundle.files {
        validate_relative_path(&file.relative_path)?;
        let path = join_under_root(out_dir, &file.relative_path).map_err(|err| {
            diagnostics::report_error(write_failed(format!(
                "unsafe binding path {}: {err}",
                file.relative_path
            )))
        })?;
        if let Some(parent) = path.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                return Err(diagnostics::report_error(write_failed(format!(
                    "failed to create directory {}: {err}",
                    parent.display()
                ))));
            }
        }
        if let Err(err) = fs::write(&path, &file.content) {
            return Err(diagnostics::report_error(write_failed(format!(
                "failed to write {}: {err}",
                path.display()
            ))));
        }
    }
    Ok(())
}
