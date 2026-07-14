//! Shared plan views used by orchestrator adapters.

use crate::binding::artifact::BindingFile;
use crate::binding::framework::BindContext;
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
    #[allow(dead_code)]
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

    /// Sanitize an identifier for use as a Python name.
    pub fn python_ident(raw: &str) -> String {
        let mut out = String::with_capacity(raw.len());
        for (i, ch) in raw.chars().enumerate() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                out.push(ch);
            } else {
                out.push('_');
            }
            if i == 0 && out.starts_with(|c: char| c.is_ascii_digit()) {
                out.insert(0, '_');
            }
        }
        if out.is_empty() {
            "_unnamed".to_owned()
        } else {
            out
        }
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
            out = out.trim_end_matches('-').to_owned();
            out
        } else {
            out
        }
    }

    /// Escape a string for embedding in a Python double-quoted literal.
    pub fn py_string(raw: &str) -> String {
        raw.replace('\\', "\\\\").replace('"', "\\\"")
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
            "# Generated by dpcs; preserve plan semantics — do not invent runtime behavior."
                .to_owned(),
        );
        lines.join("\n")
    }
}

/// Helper to build a Python source file artifact.
pub fn python_file(relative_path: &str, content: String) -> BindingFile {
    BindingFile::new(relative_path, "text/x-python", content)
}

/// Helper to build a YAML artifact.
pub fn yaml_file(relative_path: &str, content: String) -> BindingFile {
    BindingFile::new(relative_path, "application/yaml", content)
}
