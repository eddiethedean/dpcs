//! Shared plan views used by orchestrator adapters.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use crate::binding::artifact::BindingFile;
use crate::binding::diagnostics::{self, report_error};
use crate::binding::framework::BindContext;
use crate::diagnostics::ValidationReport;
use crate::model::{
    ContractReference, ExecutionRequirements, FailureSemantics, PipelineStep, QualityGate,
    SchedulingIntent, SchedulingMode,
};
use crate::plan::{PipelinePlan, PlanDependencyEdge};

/// Stable, adapter-facing projection of a Pipeline Plan.
#[derive(Debug, Clone)]
pub struct PlanView<'a> {
    /// Source plan.
    pub plan: &'a PipelinePlan,
    /// Binding context (profile + capability report).
    pub ctx: &'a BindContext<'a>,
}

impl<'a> PlanView<'a> {
    /// Create a plan view.
    pub fn new(plan: &'a PipelinePlan, ctx: &'a BindContext<'a>) -> Self {
        Self { plan, ctx }
    }

    /// Contract identifier.
    pub fn contract_id(&self) -> &str {
        &self.plan.contract_id
    }

    /// Contract version.
    pub fn contract_version(&self) -> &str {
        &self.plan.contract_version
    }

    /// Profile identity used for binding.
    pub fn profile_identity(&self) -> &str {
        self.ctx.profile_identity
    }

    /// Ordered step identifiers (plan order).
    pub fn step_order(&self) -> &[String] {
        &self.plan.step_order
    }

    /// Steps keyed by declaration with lookup by id via [`Self::step`].
    pub fn steps(&self) -> &[PipelineStep] {
        &self.plan.steps
    }

    /// Find a step by id.
    pub fn step(&self, id: &str) -> Option<&PipelineStep> {
        self.plan.steps.iter().find(|step| step.id == id)
    }

    /// Dependency edges.
    pub fn dependency_edges(&self) -> &[PlanDependencyEdge] {
        &self.plan.dependency_edges
    }

    /// Contract references.
    pub fn contract_references(&self) -> &[ContractReference] {
        &self.plan.contract_references
    }

    /// Scheduling intents.
    pub fn scheduling(&self) -> &[SchedulingIntent] {
        &self.plan.scheduling
    }

    /// Quality gates.
    pub fn quality_gates(&self) -> &[QualityGate] {
        &self.plan.quality_gates
    }

    /// Failure semantics.
    pub fn failure_semantics(&self) -> &[FailureSemantics] {
        &self.plan.failure_semantics
    }

    /// Execution requirements.
    pub fn execution(&self) -> Option<&ExecutionRequirements> {
        self.plan.execution.as_ref()
    }

    /// First cron expression from scheduled intents, if any.
    pub fn primary_cron(&self) -> Option<&str> {
        self.plan.scheduling.iter().find_map(|intent| {
            if matches!(intent.mode, SchedulingMode::Scheduled) {
                intent.cron.as_deref()
            } else {
                None
            }
        })
    }

    /// First timezone from scheduling intents, if any.
    pub fn primary_timezone(&self) -> Option<&str> {
        self.plan
            .scheduling
            .iter()
            .find_map(|intent| intent.timezone.as_deref())
    }

    /// Predecessor step ids for `step_id` derived from dependency edges (sorted).
    pub fn predecessors(&self, step_id: &str) -> Vec<&str> {
        let mut preds: Vec<&str> = self
            .dependency_edges()
            .iter()
            .filter(|edge| edge.to == step_id)
            .map(|edge| edge.from.as_str())
            .collect();
        preds.sort_unstable();
        preds.dedup();
        preds
    }

    /// Unique Python identifiers for every plan step id (and optional extras).
    pub fn unique_python_idents(&self, extras: &[&str]) -> BTreeMap<String, String> {
        let mut ids: Vec<String> = self.step_order().to_vec();
        for extra in extras {
            if !ids.iter().any(|id| id == *extra) {
                ids.push((*extra).to_owned());
            }
        }
        unique_sanitized(&ids, Self::python_ident)
    }

    /// Unique Kubernetes name fragments for every plan step id (and optional extras).
    pub fn unique_k8s_names(&self, extras: &[&str]) -> BTreeMap<String, String> {
        let mut ids: Vec<String> = self.step_order().to_vec();
        for extra in extras {
            if !ids.iter().any(|id| id == *extra) {
                ids.push((*extra).to_owned());
            }
        }
        unique_sanitized(&ids, Self::k8s_name)
    }

    /// Sanitize an identifier for use as a Python name.
    pub fn python_ident(raw: &str) -> String {
        let mut out = String::with_capacity(raw.len());
        for ch in raw.chars() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                out.push(ch);
            } else {
                out.push('_');
            }
        }
        if out.is_empty() {
            return "_unnamed".to_owned();
        }
        if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            out.insert(0, '_');
        }
        out
    }

    /// Build a PascalCase Python class name (Temporal workflow class).
    pub fn python_class_name(raw: &str) -> String {
        let mut parts = Vec::new();
        let mut current = String::new();
        for ch in raw.chars() {
            if ch.is_ascii_alphanumeric() {
                current.push(ch);
            } else if !current.is_empty() {
                parts.push(std::mem::take(&mut current));
            }
        }
        if !current.is_empty() {
            parts.push(current);
        }
        let mut name = String::new();
        for part in parts {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                name.push(first.to_ascii_uppercase());
                for ch in chars {
                    name.push(ch.to_ascii_lowercase());
                }
            }
        }
        if name.is_empty() {
            name = "Pipeline".to_owned();
        }
        if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            name.insert(0, '_');
        }
        if !name.ends_with("Workflow") {
            name.push_str("Workflow");
        }
        name
    }

    /// Sanitize an identifier for use as a Kubernetes label / resource name fragment.
    pub fn k8s_name(raw: &str) -> String {
        let mut out: String = raw
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() {
                    ch.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect();
        while out.contains("--") {
            out = out.replace("--", "-");
        }
        out = out.trim_matches('-').to_owned();
        if out.is_empty() {
            "pipeline".to_owned()
        } else if out.len() > 63 {
            out.truncate(63);
            out.trim_end_matches('-').to_owned()
        } else {
            out
        }
    }

    /// Escape a string for embedding in a Python double-quoted literal.
    pub fn py_string(raw: &str) -> String {
        let mut out = String::with_capacity(raw.len());
        for ch in raw.chars() {
            match ch {
                '\\' => out.push_str("\\\\"),
                '"' => out.push_str("\\\""),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c.is_control() => {
                    out.push_str(&format!("\\u{{{:04x}}}", u32::from(c)));
                }
                c => out.push(c),
            }
        }
        out
    }

    /// Escape a string for a YAML double-quoted scalar.
    pub fn yaml_string(raw: &str) -> String {
        let mut out = String::with_capacity(raw.len());
        for ch in raw.chars() {
            match ch {
                '\\' => out.push_str("\\\\"),
                '"' => out.push_str("\\\""),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c.is_control() => {
                    out.push_str(&format!("\\u{:04x}", u32::from(c)));
                }
                c => out.push(c),
            }
        }
        out
    }

    /// Build a shared header comment block documenting preserved semantics.
    pub fn header_comment(&self, platform: &str) -> String {
        let mut lines = vec![
            format!("# DPCS orchestrator binding scaffold ({platform})"),
            format!("# contractId: {}", self.contract_id()),
            format!("# contractVersion: {}", self.contract_version()),
            format!("# profileIdentity: {}", self.profile_identity()),
            format!("# dpcsVersion: {}", self.plan.dpcs_version),
            format!("# stepOrder: {}", self.step_order().join(", ")),
        ];
        if !self.scheduling().is_empty() {
            let modes: Vec<_> = self
                .scheduling()
                .iter()
                .map(|intent| intent.mode.as_str())
                .collect();
            lines.push(format!("# schedulingModes: {}", modes.join(", ")));
        }
        if let Some(cron) = self.primary_cron() {
            lines.push(format!("# scheduleCron: {cron}"));
        }
        if let Some(tz) = self.primary_timezone() {
            lines.push(format!("# scheduleTimezone: {tz}"));
        }
        if !self.quality_gates().is_empty() {
            let ids: Vec<_> = self.quality_gates().iter().map(|g| g.id.as_str()).collect();
            lines.push(format!("# qualityGates: {}", ids.join(", ")));
        }
        if !self.failure_semantics().is_empty() {
            let ids: Vec<_> = self
                .failure_semantics()
                .iter()
                .map(|f| f.id.as_str())
                .collect();
            lines.push(format!("# failureSemantics: {}", ids.join(", ")));
        }
        if let Some(exec) = self.execution() {
            if !exec.required_capabilities.is_empty() {
                lines.push(format!(
                    "# requiredCapabilities: {}",
                    exec.required_capabilities.join(", ")
                ));
            }
        }
        if let Some(lineage) = &self.plan.lineage {
            if let Some(prov) = &lineage.provenance {
                if let Some(originating) = &prov.originating {
                    lines.push(format!("# lineageOriginating: {originating}"));
                }
            }
        }
        for step in self.steps() {
            let contract = step
                .contract_ref
                .as_deref()
                .or(step.transform_ref.as_deref())
                .unwrap_or("-");
            lines.push(format!(
                "# step {} type={} contractRef={}",
                step.id, step.step_type, contract
            ));
        }
        for edge in self.dependency_edges() {
            lines.push(format!("# dependency {} -> {}", edge.from, edge.to));
        }
        for reference in self.contract_references() {
            lines.push(format!(
                "# contractReference {} type={} location={}",
                reference.id, reference.reference_type, reference.location
            ));
        }
        lines.push(
            "# Scaffold encodes identity/topology where the target allows; QG/FS/execution intents are documented here."
                .to_owned(),
        );
        lines.push(
            "# Generated by dpcs; do not invent runtime behavior beyond the Pipeline Plan."
                .to_owned(),
        );
        lines.join("\n")
    }
}

/// Map original ids to unique sanitized identifiers, disambiguating collisions.
pub fn unique_sanitized(
    ids: &[String],
    sanitize: impl Fn(&str) -> String,
) -> BTreeMap<String, String> {
    let mut used = BTreeSet::new();
    let mut map = BTreeMap::new();
    for id in ids {
        let base = sanitize(id);
        let mut candidate = base.clone();
        let mut n = 2u32;
        while !used.insert(candidate.clone()) {
            candidate = format!("{base}__{n}");
            n += 1;
        }
        map.insert(id.clone(), candidate);
    }
    map
}

/// Reject relative paths that can escape `out_dir`.
pub fn validate_relative_path(relative_path: &str) -> Result<(), ValidationReport> {
    if relative_path.is_empty() {
        return Err(report_error(diagnostics::write_failed(
            "binding file relative_path must not be empty",
        )));
    }
    if relative_path.contains('\0') {
        return Err(report_error(diagnostics::write_failed(
            "binding file relative_path must not contain NUL",
        )));
    }
    let path = Path::new(relative_path);
    if path.is_absolute() {
        return Err(report_error(diagnostics::write_failed(format!(
            "binding file relative_path must be relative, got `{relative_path}`"
        ))));
    }
    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir => {
                return Err(report_error(diagnostics::write_failed(format!(
                    "binding file relative_path must not contain `..`: `{relative_path}`"
                ))));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(report_error(diagnostics::write_failed(format!(
                    "binding file relative_path must be relative, got `{relative_path}`"
                ))));
            }
        }
    }
    Ok(())
}

/// Helper to build a Python source file artifact.
pub fn python_file(relative_path: &str, content: String) -> BindingFile {
    BindingFile::new(relative_path, "text/x-python", content)
}

/// Helper to build a YAML artifact.
pub fn yaml_file(relative_path: &str, content: String) -> BindingFile {
    BindingFile::new(relative_path, "application/yaml", content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_sanitized_disambiguates_collisions() {
        let ids = vec!["a.b".to_owned(), "a_b".to_owned()];
        let map = unique_sanitized(&ids, PlanView::python_ident);
        assert_ne!(map["a.b"], map["a_b"]);
        assert!(map.values().all(|v| v.starts_with("a_b")));
    }

    #[test]
    fn validate_relative_path_rejects_escape() {
        assert!(validate_relative_path("../etc/passwd").is_err());
        assert!(validate_relative_path("/etc/passwd").is_err());
        assert!(validate_relative_path("dags/ok.py").is_ok());
    }

    #[test]
    fn python_class_name_is_pascal_case() {
        assert_eq!(
            PlanView::python_class_name("valid.execution.model"),
            "ValidExecutionModelWorkflow"
        );
    }
}
