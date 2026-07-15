//! Deterministic compatibility evaluation (SPEC Ch 19).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{
    ExecutionRequirements, FailureSemantics, InterfacePort, PipelineContract, PipelineStep,
    QualityGate, SchedulingIntent, StepPort,
};
use crate::plan::PipelinePlan;

/// Compatibility category between two artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum CompatibilityCategory {
    /// No semantic differences in considered fields.
    FullyCompatible,
    /// Candidate preserves baseline observable semantics (additive OK).
    BackwardCompatible,
    /// Candidate removes optional content while preserving core semantics.
    ForwardCompatible,
    /// Compatible only under declared conditions / warnings.
    ConditionallyCompatible,
    /// Mandatory observable semantics diverge.
    Incompatible,
}

impl CompatibilityCategory {
    /// Returns `true` when the category is considered a successful match.
    pub fn is_compatible(self) -> bool {
        !matches!(self, Self::Incompatible)
    }
}

impl std::fmt::Display for CompatibilityCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FullyCompatible => write!(f, "fullyCompatible"),
            Self::BackwardCompatible => write!(f, "backwardCompatible"),
            Self::ForwardCompatible => write!(f, "forwardCompatible"),
            Self::ConditionallyCompatible => write!(f, "conditionallyCompatible"),
            Self::Incompatible => write!(f, "incompatible"),
        }
    }
}

/// Structured compatibility report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityReport {
    /// Baseline artifact identity.
    pub baseline_id: String,
    /// Baseline artifact version.
    pub baseline_version: String,
    /// Candidate artifact identity.
    pub candidate_id: String,
    /// Candidate artifact version.
    pub candidate_version: String,
    /// Resulting compatibility category.
    pub category: CompatibilityCategory,
    /// Breaking-change findings and related diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

/// Result of compatibility evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum CompatibilityResult {
    /// Compatible (fully / backward / forward / conditional).
    Ok(Box<CompatibilityReport>),
    /// Incompatible or analysis errors.
    Err {
        /// Structured report retaining category and detail.
        report: Box<CompatibilityReport>,
        /// Diagnostics for CLI / tooling.
        diagnostics: ValidationReport,
    },
}

impl CompatibilityResult {
    /// Returns whether evaluation concluded a compatible category.
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    /// Returns the compatibility report in either outcome.
    pub fn report(&self) -> &CompatibilityReport {
        match self {
            Self::Ok(report) => report,
            Self::Err { report, .. } => report,
        }
    }
}

/// Counters used to classify compatibility.
#[derive(Default)]
struct CompatCounters {
    breaking: usize,
    additive: usize,
    removed_optional: usize,
    conditional: usize,
}

/// Compare two Pipeline Contracts for semantic compatibility.
///
/// Version identifiers alone never decide the category.
pub fn compare_contracts(
    baseline: &PipelineContract,
    candidate: &PipelineContract,
) -> CompatibilityResult {
    let mut diagnostics = ValidationReport::new();
    let mut counters = CompatCounters::default();

    if baseline.id != candidate.id {
        diagnostics.push(
            Diagnostic::compatibility_warning(
                "DPCS-COMPAT-001",
                categories::COMPATIBILITY,
                format!(
                    "pipeline identity differs (`{}` vs `{}`); comparison continues on structure",
                    baseline.id, candidate.id
                ),
            )
            .with_object_ref("id"),
        );
        counters.conditional += 1;
    }

    compare_ports(
        "interface.inputs",
        &baseline.interface.inputs,
        &candidate.interface.inputs,
        &mut diagnostics,
        &mut counters,
    );
    compare_ports(
        "interface.outputs",
        &baseline.interface.outputs,
        &candidate.interface.outputs,
        &mut diagnostics,
        &mut counters,
    );

    compare_steps(
        &baseline.steps,
        &candidate.steps,
        &mut diagnostics,
        &mut counters,
    );

    compare_string_pairs(
        "graph.edges",
        edge_set(baseline),
        edge_set(candidate),
        &mut diagnostics,
        &mut counters,
    );
    compare_string_pairs(
        "dataFlow",
        flow_set(baseline),
        flow_set(candidate),
        &mut diagnostics,
        &mut counters,
    );
    compare_string_pairs(
        "controlFlow",
        control_set(baseline),
        control_set(candidate),
        &mut diagnostics,
        &mut counters,
    );

    compare_refs(baseline, candidate, &mut diagnostics, &mut counters);

    compare_execution(
        baseline.execution.as_ref(),
        candidate.execution.as_ref(),
        &mut diagnostics,
        &mut counters,
    );
    compare_scheduling(
        &baseline.scheduling,
        &candidate.scheduling,
        &mut diagnostics,
        &mut counters,
    );
    compare_quality_gates(
        &baseline.quality_gates,
        &candidate.quality_gates,
        &mut diagnostics,
        &mut counters,
    );
    compare_failure_semantics(
        &baseline.failure_semantics,
        &candidate.failure_semantics,
        &mut diagnostics,
        &mut counters,
    );

    match (&baseline.lineage, &candidate.lineage) {
        (None, None) => {}
        (Some(_), None) => {
            diagnostics.push(breaking_diag(
                "DPCS-COMPAT-028",
                "lineage removed",
                "lineage",
            ));
            counters.breaking += 1;
        }
        (None, Some(_)) => {
            counters.additive += 1;
        }
        (Some(a), Some(b)) => {
            if lineage_fingerprint(a) != lineage_fingerprint(b) {
                diagnostics.push(breaking_diag(
                    "DPCS-COMPAT-029",
                    "lineage changed incompatibly",
                    "lineage",
                ));
                counters.breaking += 1;
            }
        }
    }

    let baseline_ext: BTreeSet<_> = baseline.extensions.keys().cloned().collect();
    let candidate_ext: BTreeSet<_> = candidate.extensions.keys().cloned().collect();
    for key in baseline_ext.difference(&candidate_ext) {
        diagnostics.push(
            Diagnostic::compatibility_warning(
                "DPCS-COMPAT-030",
                categories::COMPATIBILITY,
                format!("extension `{key}` removed"),
            )
            .with_object_ref(key.as_str()),
        );
        // Extension removals are optional content → forward-compatible when alone.
        counters.removed_optional += 1;
    }
    for _key in candidate_ext.difference(&baseline_ext) {
        counters.additive += 1;
    }

    diagnostics.sort_deterministic();
    let category = classify(&counters);
    finish(
        baseline.id.clone(),
        baseline.version.clone(),
        candidate.id.clone(),
        candidate.version.clone(),
        category,
        diagnostics,
    )
}

/// Compare two Pipeline Plans for dependency / step-order compatibility.
pub fn compare_plans(baseline: &PipelinePlan, candidate: &PipelinePlan) -> CompatibilityResult {
    let mut diagnostics = ValidationReport::new();
    let mut counters = CompatCounters::default();

    if baseline.contract_id != candidate.contract_id {
        diagnostics.push(
            Diagnostic::compatibility_warning(
                "DPCS-COMPAT-040",
                categories::COMPATIBILITY,
                "plan contract ids differ",
            )
            .with_object_ref("contractId"),
        );
        counters.conditional += 1;
    }

    let baseline_steps: BTreeSet<_> = baseline.steps.iter().map(|s| s.id.clone()).collect();
    let candidate_steps: BTreeSet<_> = candidate.steps.iter().map(|s| s.id.clone()).collect();
    let steps_changed = baseline_steps != candidate_steps;

    for id in baseline_steps.difference(&candidate_steps) {
        diagnostics.push(breaking_diag(
            "DPCS-COMPAT-041",
            format!("plan step `{id}` removed"),
            format!("steps.{id}"),
        ));
        counters.breaking += 1;
    }
    for id in candidate_steps.difference(&baseline_steps) {
        counters.additive += 1;
        diagnostics.push(info_additive(
            "DPCS-COMPAT-042",
            format!("plan step `{id}` added"),
            format!("steps.{id}"),
        ));
    }

    let baseline_edges: BTreeSet<_> = baseline
        .dependency_edges
        .iter()
        .map(|e| format!("{}->{}", e.from, e.to))
        .collect();
    let candidate_edges: BTreeSet<_> = candidate
        .dependency_edges
        .iter()
        .map(|e| format!("{}->{}", e.from, e.to))
        .collect();
    let edges_changed = baseline_edges != candidate_edges;

    for edge in baseline_edges.difference(&candidate_edges) {
        diagnostics.push(breaking_diag(
            "DPCS-COMPAT-043",
            format!("dependency edge `{edge}` removed"),
            "dependencyEdges",
        ));
        counters.breaking += 1;
    }
    for _edge in candidate_edges.difference(&baseline_edges) {
        counters.additive += 1;
    }

    // Only flag stepOrder when step set and edges are unchanged (pure reorder).
    if baseline.step_order != candidate.step_order
        && counters.breaking == 0
        && !steps_changed
        && !edges_changed
    {
        diagnostics.push(
            Diagnostic::compatibility_warning(
                "DPCS-COMPAT-044",
                categories::COMPATIBILITY,
                "stepOrder differs while step set and dependency edges remain equal",
            )
            .with_object_ref("stepOrder"),
        );
        counters.conditional += 1;
    }

    diagnostics.sort_deterministic();
    let category = classify(&counters);
    finish(
        baseline.contract_id.clone(),
        baseline.contract_version.clone(),
        candidate.contract_id.clone(),
        candidate.contract_version.clone(),
        category,
        diagnostics,
    )
}

fn finish(
    baseline_id: String,
    baseline_version: String,
    candidate_id: String,
    candidate_version: String,
    category: CompatibilityCategory,
    diagnostics: ValidationReport,
) -> CompatibilityResult {
    let report = CompatibilityReport {
        baseline_id,
        baseline_version,
        candidate_id,
        candidate_version,
        category,
        diagnostics: diagnostics.diagnostics.clone(),
    };
    if category.is_compatible() {
        CompatibilityResult::Ok(Box::new(report))
    } else {
        CompatibilityResult::Err {
            report: Box::new(report),
            diagnostics,
        }
    }
}

fn classify(counters: &CompatCounters) -> CompatibilityCategory {
    if counters.breaking > 0 {
        CompatibilityCategory::Incompatible
    } else if counters.conditional > 0 {
        CompatibilityCategory::ConditionallyCompatible
    } else if counters.additive > 0 && counters.removed_optional == 0 {
        CompatibilityCategory::BackwardCompatible
    } else if counters.removed_optional > 0 && counters.additive == 0 {
        CompatibilityCategory::ForwardCompatible
    } else if counters.additive > 0 && counters.removed_optional > 0 {
        CompatibilityCategory::ConditionallyCompatible
    } else {
        CompatibilityCategory::FullyCompatible
    }
}

fn compare_ports(
    path: &str,
    baseline: &[InterfacePort],
    candidate: &[InterfacePort],
    diagnostics: &mut ValidationReport,
    counters: &mut CompatCounters,
) {
    if has_duplicate_ids(baseline.iter().map(|p| p.id.as_str())) {
        diagnostics.push(breaking_diag(
            "DPCS-COMPAT-012",
            format!("baseline `{path}` contains duplicate port ids"),
            path,
        ));
        counters.breaking += 1;
    }
    if has_duplicate_ids(candidate.iter().map(|p| p.id.as_str())) {
        diagnostics.push(breaking_diag(
            "DPCS-COMPAT-012",
            format!("candidate `{path}` contains duplicate port ids"),
            path,
        ));
        counters.breaking += 1;
    }

    let baseline_map: BTreeMap<_, _> = baseline.iter().map(|p| (p.id.clone(), p)).collect();
    let candidate_map: BTreeMap<_, _> = candidate.iter().map(|p| (p.id.clone(), p)).collect();

    for (id, port) in &baseline_map {
        match candidate_map.get(id) {
            None => {
                diagnostics.push(breaking_diag(
                    "DPCS-COMPAT-002",
                    format!("interface port `{id}` removed"),
                    format!("{path}.{id}"),
                ));
                counters.breaking += 1;
            }
            Some(other) => {
                if port.contract_ref != other.contract_ref {
                    diagnostics.push(breaking_diag(
                        "DPCS-COMPAT-003",
                        format!("interface port `{id}` contractRef changed"),
                        format!("{path}.{id}.contractRef"),
                    ));
                    counters.breaking += 1;
                }
                if port.name != other.name {
                    diagnostics.push(breaking_diag(
                        "DPCS-COMPAT-013",
                        format!("interface port `{id}` name changed"),
                        format!("{path}.{id}.name"),
                    ));
                    counters.breaking += 1;
                }
                if port.purpose != other.purpose {
                    diagnostics.push(breaking_diag(
                        "DPCS-COMPAT-014",
                        format!("interface port `{id}` purpose changed"),
                        format!("{path}.{id}.purpose"),
                    ));
                    counters.breaking += 1;
                }
            }
        }
    }
    for id in candidate_map.keys() {
        if !baseline_map.contains_key(id) {
            counters.additive += 1;
            diagnostics.push(info_additive(
                "DPCS-COMPAT-004",
                format!("interface port `{id}` added"),
                format!("{path}.{id}"),
            ));
        }
    }
}

fn compare_steps(
    baseline: &[PipelineStep],
    candidate: &[PipelineStep],
    diagnostics: &mut ValidationReport,
    counters: &mut CompatCounters,
) {
    if has_duplicate_ids(baseline.iter().map(|s| s.id.as_str()))
        || has_duplicate_ids(candidate.iter().map(|s| s.id.as_str()))
    {
        diagnostics.push(breaking_diag(
            "DPCS-COMPAT-015",
            "duplicate step ids prevent reliable compatibility analysis",
            "steps",
        ));
        counters.breaking += 1;
    }

    let baseline_map: BTreeMap<_, _> = baseline.iter().map(|s| (s.id.clone(), s)).collect();
    let candidate_map: BTreeMap<_, _> = candidate.iter().map(|s| (s.id.clone(), s)).collect();

    for (id, step) in &baseline_map {
        match candidate_map.get(id) {
            None => {
                diagnostics.push(breaking_diag(
                    "DPCS-COMPAT-005",
                    format!("step `{id}` removed"),
                    format!("steps.{id}"),
                ));
                counters.breaking += 1;
            }
            Some(other) => {
                if step.step_type != other.step_type {
                    diagnostics.push(breaking_diag(
                        "DPCS-COMPAT-006",
                        format!("step `{id}` type changed"),
                        format!("steps.{id}.type"),
                    ));
                    counters.breaking += 1;
                }
                if step.contract_ref != other.contract_ref {
                    diagnostics.push(breaking_diag(
                        "DPCS-COMPAT-007",
                        format!("step `{id}` contractRef changed"),
                        format!("steps.{id}.contractRef"),
                    ));
                    counters.breaking += 1;
                }
                if step.transform_ref != other.transform_ref {
                    diagnostics.push(breaking_diag(
                        "DPCS-COMPAT-016",
                        format!("step `{id}` transformRef changed"),
                        format!("steps.{id}.transformRef"),
                    ));
                    counters.breaking += 1;
                }
                compare_step_ports(
                    id,
                    "inputs",
                    &step.inputs,
                    &other.inputs,
                    diagnostics,
                    counters,
                );
                compare_step_ports(
                    id,
                    "outputs",
                    &step.outputs,
                    &other.outputs,
                    diagnostics,
                    counters,
                );
            }
        }
    }
    for id in candidate_map.keys() {
        if !baseline_map.contains_key(id) {
            counters.additive += 1;
            diagnostics.push(info_additive(
                "DPCS-COMPAT-008",
                format!("step `{id}` added"),
                format!("steps.{id}"),
            ));
        }
    }
}

fn compare_step_ports(
    step_id: &str,
    side: &str,
    baseline: &[StepPort],
    candidate: &[StepPort],
    diagnostics: &mut ValidationReport,
    counters: &mut CompatCounters,
) {
    let baseline_map: BTreeMap<_, _> = baseline.iter().map(|p| (p.id.clone(), p)).collect();
    let candidate_map: BTreeMap<_, _> = candidate.iter().map(|p| (p.id.clone(), p)).collect();
    for (id, port) in &baseline_map {
        match candidate_map.get(id) {
            None => {
                diagnostics.push(breaking_diag(
                    "DPCS-COMPAT-017",
                    format!("step `{step_id}` {side} port `{id}` removed"),
                    format!("steps.{step_id}.{side}.{id}"),
                ));
                counters.breaking += 1;
            }
            Some(other) if port.contract_ref != other.contract_ref => {
                diagnostics.push(breaking_diag(
                    "DPCS-COMPAT-018",
                    format!("step `{step_id}` {side} port `{id}` contractRef changed"),
                    format!("steps.{step_id}.{side}.{id}.contractRef"),
                ));
                counters.breaking += 1;
            }
            _ => {}
        }
    }
    for id in candidate_map.keys() {
        if !baseline_map.contains_key(id) {
            counters.additive += 1;
        }
    }
}

fn compare_string_pairs(
    path: &str,
    baseline: BTreeSet<String>,
    candidate: BTreeSet<String>,
    diagnostics: &mut ValidationReport,
    counters: &mut CompatCounters,
) {
    for edge in baseline.difference(&candidate) {
        diagnostics.push(breaking_diag(
            "DPCS-COMPAT-009",
            format!("`{path}` relation `{edge}` removed"),
            path,
        ));
        counters.breaking += 1;
    }
    for _edge in candidate.difference(&baseline) {
        counters.additive += 1;
    }
}

fn compare_refs(
    baseline: &PipelineContract,
    candidate: &PipelineContract,
    diagnostics: &mut ValidationReport,
    counters: &mut CompatCounters,
) {
    let baseline_map: BTreeMap<_, _> = baseline
        .contract_references
        .iter()
        .map(|r| (r.id.clone(), r))
        .collect();
    let candidate_map: BTreeMap<_, _> = candidate
        .contract_references
        .iter()
        .map(|r| (r.id.clone(), r))
        .collect();

    for (id, reference) in &baseline_map {
        match candidate_map.get(id) {
            None => {
                diagnostics.push(breaking_diag(
                    "DPCS-COMPAT-010",
                    format!("contract reference `{id}` removed"),
                    format!("contractReferences.{id}"),
                ));
                counters.breaking += 1;
            }
            Some(other) => {
                if reference.reference_type != other.reference_type
                    || reference.location != other.location
                {
                    diagnostics.push(breaking_diag(
                        "DPCS-COMPAT-011",
                        format!("contract reference `{id}` type/location changed"),
                        format!("contractReferences.{id}"),
                    ));
                    counters.breaking += 1;
                }
            }
        }
    }
    for id in candidate_map.keys() {
        if !baseline_map.contains_key(id) {
            counters.additive += 1;
        }
    }
}

fn compare_execution(
    baseline: Option<&ExecutionRequirements>,
    candidate: Option<&ExecutionRequirements>,
    diagnostics: &mut ValidationReport,
    counters: &mut CompatCounters,
) {
    match (baseline, candidate) {
        (None, None) => {}
        (Some(_), None) => {
            diagnostics.push(breaking_diag(
                "DPCS-COMPAT-020",
                "execution requirements removed",
                "execution",
            ));
            counters.breaking += 1;
        }
        (None, Some(_)) => {
            diagnostics.push(info_additive(
                "DPCS-COMPAT-021",
                "execution requirements added",
                "execution",
            ));
            counters.additive += 1;
        }
        (Some(a), Some(b)) => {
            if execution_fingerprint(a) != execution_fingerprint(b) {
                diagnostics.push(breaking_diag(
                    "DPCS-COMPAT-022",
                    "execution requirements changed incompatibly",
                    "execution",
                ));
                counters.breaking += 1;
            }
        }
    }
}

fn compare_scheduling(
    baseline: &[SchedulingIntent],
    candidate: &[SchedulingIntent],
    diagnostics: &mut ValidationReport,
    counters: &mut CompatCounters,
) {
    let baseline_set = scheduling_fingerprints(baseline);
    let candidate_set = scheduling_fingerprints(candidate);
    if baseline_set == candidate_set {
        return;
    }
    if candidate.is_empty() && !baseline.is_empty() {
        diagnostics.push(breaking_diag(
            "DPCS-COMPAT-023",
            "scheduling intents removed",
            "scheduling",
        ));
        counters.breaking += 1;
    } else if baseline.is_empty() && !candidate.is_empty() {
        counters.additive += 1;
        diagnostics.push(info_additive(
            "DPCS-COMPAT-024",
            "scheduling intents added",
            "scheduling",
        ));
    } else {
        diagnostics.push(breaking_diag(
            "DPCS-COMPAT-025",
            "scheduling intents changed incompatibly",
            "scheduling",
        ));
        counters.breaking += 1;
    }
}

fn compare_quality_gates(
    baseline: &[QualityGate],
    candidate: &[QualityGate],
    diagnostics: &mut ValidationReport,
    counters: &mut CompatCounters,
) {
    if quality_fingerprints(baseline) != quality_fingerprints(candidate) {
        diagnostics.push(breaking_diag(
            "DPCS-COMPAT-026",
            "quality gates changed",
            "qualityGates",
        ));
        counters.breaking += 1;
    }
}

fn compare_failure_semantics(
    baseline: &[FailureSemantics],
    candidate: &[FailureSemantics],
    diagnostics: &mut ValidationReport,
    counters: &mut CompatCounters,
) {
    if failure_fingerprints(baseline) != failure_fingerprints(candidate) {
        diagnostics.push(breaking_diag(
            "DPCS-COMPAT-027",
            "failure semantics changed",
            "failureSemantics",
        ));
        counters.breaking += 1;
    }
}

fn execution_fingerprint(execution: &ExecutionRequirements) -> String {
    let caps: BTreeSet<_> = execution
        .required_capabilities
        .iter()
        .map(|c| c.trim().to_owned())
        .collect();
    let isolation: BTreeSet<_> = execution.isolation.iter().cloned().collect();
    let deps: BTreeSet<_> = execution
        .external_dependencies
        .iter()
        .map(|d| format!("{}:{}", d.id, d.capability))
        .collect();
    let resources = execution.resources.as_ref().map(|r| {
        format!(
            "{:?}/{:?}/{:?}/{:?}/{:?}",
            r.processor, r.memory, r.storage, r.accelerator, r.bandwidth
        )
    });
    let environment = execution.environment.as_ref().map(|e| {
        let runtime: BTreeSet<_> = e.runtime_dependencies.iter().cloned().collect();
        let software: BTreeSet<_> = e.software_capabilities.iter().cloned().collect();
        format!(
            "{:?}/{:?}/{:?}/{:?}",
            e.operating_system, runtime, software, e.container
        )
    });
    format!("{caps:?}|{isolation:?}|{deps:?}|{resources:?}|{environment:?}")
}

fn scheduling_fingerprints(items: &[SchedulingIntent]) -> BTreeSet<String> {
    items
        .iter()
        .map(|s| {
            let events: BTreeSet<_> = s
                .events
                .iter()
                .map(|e| format!("{}|{}|{:?}", e.id, e.source, e.condition))
                .collect();
            let windows: BTreeSet<_> = s.windows.iter().cloned().collect();
            let blackouts: BTreeSet<_> = s.blackouts.iter().cloned().collect();
            let deadlines: BTreeSet<_> = s.deadlines.iter().cloned().collect();
            let policies: BTreeSet<_> = s.policies.iter().cloned().collect();
            let constraints = s.constraints.as_ref().map(|c| {
                format!(
                    "{:?}|{:?}|{:?}|{:?}|{:?}",
                    c.earliest, c.latest, c.deadline, c.concurrency, c.ordering
                )
            });
            format!(
                "{:?}|{:?}|{:?}|{:?}|{:?}|{windows:?}|{blackouts:?}|{deadlines:?}|{events:?}|{constraints:?}|{policies:?}",
                s.id, s.mode, s.cron, s.timezone, s.frequency
            )
        })
        .collect()
}

fn quality_fingerprints(items: &[QualityGate]) -> BTreeSet<String> {
    items
        .iter()
        .map(|g| {
            let criteria: BTreeSet<_> = g
                .criteria
                .iter()
                .map(|c| {
                    format!(
                        "{:?}|{:?}|{:?}|{:?}",
                        c.id, c.criterion_type, c.contract_ref, c.expression
                    )
                })
                .collect();
            format!(
                "{}|{}|{:?}|{:?}|{:?}|{:?}|{criteria:?}",
                g.id, g.purpose, g.on_success, g.on_failure, g.category, g.placement
            )
        })
        .collect()
}

fn failure_fingerprints(items: &[FailureSemantics]) -> BTreeSet<String> {
    items
        .iter()
        .map(|f| {
            let triggers: BTreeSet<_> = f.triggers.iter().cloned().collect();
            let responses: BTreeSet<_> =
                f.responses.iter().map(|r| r.as_str().to_owned()).collect();
            let retry = f.retry.as_ref().map(|r| {
                format!(
                    "{:?}|{:?}|{:?}|{:?}|{:?}",
                    r.eligible, r.max_attempts, r.conditions, r.delay_policy, r.termination
                )
            });
            let recovery = f.recovery.as_ref().map(|r| {
                format!(
                    "{:?}|{:?}|{:?}|{:?}|{:?}",
                    r.restart, r.resume, r.rollback, r.checkpoint, r.state_restoration
                )
            });
            format!(
                "{}|{:?}|{triggers:?}|{responses:?}|{:?}|{retry:?}|{:?}|{recovery:?}",
                f.id, f.scope, f.category, f.compensation
            )
        })
        .collect()
}

fn lineage_fingerprint(lineage: &crate::model::PipelineLineage) -> String {
    let datasets: BTreeSet<_> = lineage
        .datasets
        .iter()
        .map(|d| {
            let consumed: BTreeSet<_> = d.consumed_by.iter().cloned().collect();
            format!(
                "{}|{:?}|{consumed:?}|{:?}|{:?}",
                d.dataset, d.produced_by, d.contract_ref, d.transform_ref
            )
        })
        .collect();
    let steps: BTreeSet<_> = lineage
        .steps
        .iter()
        .map(|s| {
            let pred: BTreeSet<_> = s.predecessors.iter().cloned().collect();
            let succ: BTreeSet<_> = s.successors.iter().cloned().collect();
            format!(
                "{}|{pred:?}|{succ:?}|{:?}|{:?}",
                s.step_id, s.dependency_kind, s.contract_ref
            )
        })
        .collect();
    let provenance = lineage.provenance.as_ref().map(|p| {
        let parents: BTreeSet<_> = p.parents.iter().cloned().collect();
        let nested: BTreeSet<_> = p.nested.iter().cloned().collect();
        let imported: BTreeSet<_> = p.imported.iter().cloned().collect();
        let history: BTreeSet<_> = p.version_history.iter().cloned().collect();
        format!(
            "{:?}|{parents:?}|{nested:?}|{imported:?}|{history:?}",
            p.originating
        )
    });
    format!("{datasets:?}|{steps:?}|{provenance:?}")
}

fn edge_set(contract: &PipelineContract) -> BTreeSet<String> {
    contract
        .graph
        .edges
        .iter()
        .map(|e| format!("{}->{}:{}", e.from, e.to, e.kind.as_deref().unwrap_or("")))
        .collect()
}

fn flow_set(contract: &PipelineContract) -> BTreeSet<String> {
    contract
        .data_flow
        .iter()
        .map(|f| {
            format!(
                "{}=>{}:{}",
                f.from,
                f.to,
                f.dataset.as_deref().unwrap_or("")
            )
        })
        .collect()
}

fn control_set(contract: &PipelineContract) -> BTreeSet<String> {
    contract
        .control_flow
        .iter()
        .map(|f| format!("{}=>{}:{}", f.from, f.to, f.kind.as_deref().unwrap_or("")))
        .collect()
}

fn has_duplicate_ids<'a>(ids: impl Iterator<Item = &'a str>) -> bool {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            return true;
        }
    }
    false
}

fn breaking_diag(
    id: &str,
    message: impl Into<String>,
    object_ref: impl Into<String>,
) -> Diagnostic {
    Diagnostic::compatibility_error(id, categories::COMPATIBILITY, message)
        .with_object_ref(object_ref)
        .with_remediation("Treat as a breaking change and bump accordingly")
}

fn info_additive(
    id: &str,
    message: impl Into<String>,
    object_ref: impl Into<String>,
) -> Diagnostic {
    Diagnostic {
        id: id.into(),
        severity: crate::diagnostics::Severity::Information,
        stage: crate::diagnostics::DiagnosticStage::CompatibilityAnalysis,
        category: categories::COMPATIBILITY.to_owned(),
        message: message.into(),
        object_ref: Some(object_ref.into()),
        remediation: Some("Additive changes are typically backward compatible".to_owned()),
        source_location: None,
        related_diagnostics: Vec::new(),
        metadata: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_yaml;
    use crate::plan::{plan, PlanResult};

    fn contract(yaml: &str) -> PipelineContract {
        parse_yaml(yaml).unwrap()
    }

    #[test]
    fn additive_step_is_backward_compatible() {
        let baseline = contract(
            r#"
dpcsVersion: "1.0.0"
id: "p"
version: "1.0.0"
interface:
  inputs: []
  outputs: []
steps:
  - id: "a"
    type: "extension:noop"
graph:
  edges: []
"#,
        );
        let candidate = contract(
            r#"
dpcsVersion: "1.0.0"
id: "p"
version: "1.1.0"
interface:
  inputs: []
  outputs: []
steps:
  - id: "a"
    type: "extension:noop"
  - id: "b"
    type: "extension:noop"
graph:
  edges: []
"#,
        );
        let result = compare_contracts(&baseline, &candidate);
        assert!(result.is_ok());
        assert_eq!(
            result.report().category,
            CompatibilityCategory::BackwardCompatible
        );
    }

    #[test]
    fn step_port_removal_is_breaking() {
        let baseline = contract(
            r#"
dpcsVersion: "1.0.0"
id: "p"
version: "1.0.0"
interface:
  inputs: []
  outputs: []
steps:
  - id: "a"
    type: "extension:noop"
    inputs:
      - id: "in"
graph:
  edges: []
"#,
        );
        let candidate = contract(
            r#"
dpcsVersion: "1.0.0"
id: "p"
version: "1.1.0"
interface:
  inputs: []
  outputs: []
steps:
  - id: "a"
    type: "extension:noop"
graph:
  edges: []
"#,
        );
        let result = compare_contracts(&baseline, &candidate);
        assert!(!result.is_ok());
        assert!(result
            .report()
            .diagnostics
            .iter()
            .any(|d| d.id == "DPCS-COMPAT-017"));
    }

    #[test]
    fn control_flow_kind_change_is_breaking() {
        let baseline = contract(
            r#"
dpcsVersion: "1.0.0"
id: "p"
version: "1.0.0"
interface:
  inputs: []
  outputs: []
steps:
  - id: "a"
    type: "extension:noop"
  - id: "b"
    type: "extension:noop"
graph:
  edges: []
controlFlow:
  - from: "a"
    to: "b"
    kind: "success"
"#,
        );
        let candidate = contract(
            r#"
dpcsVersion: "1.0.0"
id: "p"
version: "1.1.0"
interface:
  inputs: []
  outputs: []
steps:
  - id: "a"
    type: "extension:noop"
  - id: "b"
    type: "extension:noop"
graph:
  edges: []
controlFlow:
  - from: "a"
    to: "b"
    kind: "failure"
"#,
        );
        let result = compare_contracts(&baseline, &candidate);
        assert!(!result.is_ok());
    }

    #[test]
    fn extension_removal_is_forward_compatible() {
        let baseline = contract(
            r#"
dpcsVersion: "1.0.0"
id: "p"
version: "1.0.0"
interface:
  inputs: []
  outputs: []
steps:
  - id: "a"
    type: "extension:noop"
graph:
  edges: []
x-vendor:
  team: data
"#,
        );
        let candidate = contract(
            r#"
dpcsVersion: "1.0.0"
id: "p"
version: "1.1.0"
interface:
  inputs: []
  outputs: []
steps:
  - id: "a"
    type: "extension:noop"
graph:
  edges: []
"#,
        );
        let result = compare_contracts(&baseline, &candidate);
        assert!(result.is_ok());
        assert_eq!(
            result.report().category,
            CompatibilityCategory::ForwardCompatible
        );
    }

    #[test]
    fn compare_plans_additive_is_backward_compatible() {
        let baseline = contract(
            r#"
dpcsVersion: "1.0.0"
id: "p"
version: "1.0.0"
interface:
  inputs: []
  outputs: []
steps:
  - id: "a"
    type: "extension:noop"
graph:
  edges: []
"#,
        );
        let candidate = contract(
            r#"
dpcsVersion: "1.0.0"
id: "p"
version: "1.1.0"
interface:
  inputs: []
  outputs: []
steps:
  - id: "a"
    type: "extension:noop"
  - id: "b"
    type: "extension:noop"
graph:
  edges: []
"#,
        );
        let PlanResult::Ok(base_plan) = plan(&baseline) else {
            panic!("baseline plan");
        };
        let PlanResult::Ok(cand_plan) = plan(&candidate) else {
            panic!("candidate plan");
        };
        let result = compare_plans(&base_plan, &cand_plan);
        assert!(result.is_ok());
        assert_eq!(
            result.report().category,
            CompatibilityCategory::BackwardCompatible
        );
    }
}
