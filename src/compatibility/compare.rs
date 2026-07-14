//! Deterministic compatibility evaluation (SPEC Ch 19).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{InterfacePort, PipelineContract, PipelineStep};
use crate::plan::PipelinePlan;

/// Compatibility category between two artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompatibilityCategory {
    /// No semantic differences in considered fields.
    FullyCompatible,
    /// Candidate preserves baseline observable semantics (additive OK).
    BackwardCompatible,
    /// Baseline preserves candidate semantics (unusual; candidate removes optional).
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

/// Compare two Pipeline Contracts for semantic compatibility.
///
/// Version identifiers alone never decide the category.
pub fn compare_contracts(
    baseline: &PipelineContract,
    candidate: &PipelineContract,
) -> CompatibilityResult {
    let mut diagnostics = ValidationReport::new();
    let mut breaking = 0usize;
    let mut additive = 0usize;
    let mut removed_optional = 0usize;
    let mut conditional = 0usize;

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
        conditional += 1;
    }

    compare_ports(
        "interface.inputs",
        &baseline.interface.inputs,
        &candidate.interface.inputs,
        &mut diagnostics,
        &mut breaking,
        &mut additive,
        &mut removed_optional,
    );
    compare_ports(
        "interface.outputs",
        &baseline.interface.outputs,
        &candidate.interface.outputs,
        &mut diagnostics,
        &mut breaking,
        &mut additive,
        &mut removed_optional,
    );

    compare_steps(
        &baseline.steps,
        &candidate.steps,
        &mut diagnostics,
        &mut breaking,
        &mut additive,
        &mut removed_optional,
    );

    compare_string_pairs(
        "graph.edges",
        edge_set(baseline),
        edge_set(candidate),
        &mut diagnostics,
        &mut breaking,
        &mut additive,
        &mut removed_optional,
    );
    compare_string_pairs(
        "dataFlow",
        flow_set(baseline),
        flow_set(candidate),
        &mut diagnostics,
        &mut breaking,
        &mut additive,
        &mut removed_optional,
    );
    compare_string_pairs(
        "controlFlow",
        control_set(baseline),
        control_set(candidate),
        &mut diagnostics,
        &mut breaking,
        &mut additive,
        &mut removed_optional,
    );

    compare_refs(
        baseline,
        candidate,
        &mut diagnostics,
        &mut breaking,
        &mut additive,
        &mut removed_optional,
    );

    if baseline.execution != candidate.execution {
        if baseline.execution.is_some() && candidate.execution.is_none() {
            diagnostics.push(breaking_diag(
                "DPCS-COMPAT-020",
                "execution requirements removed",
                "execution",
            ));
            breaking += 1;
        } else if baseline.execution.is_none() && candidate.execution.is_some() {
            diagnostics.push(info_additive(
                "DPCS-COMPAT-021",
                "execution requirements added",
                "execution",
            ));
            additive += 1;
        } else {
            diagnostics.push(breaking_diag(
                "DPCS-COMPAT-022",
                "execution requirements changed incompatibly",
                "execution",
            ));
            breaking += 1;
        }
    }

    if baseline.scheduling != candidate.scheduling {
        if candidate.scheduling.is_empty() && !baseline.scheduling.is_empty() {
            diagnostics.push(breaking_diag(
                "DPCS-COMPAT-023",
                "scheduling intents removed",
                "scheduling",
            ));
            breaking += 1;
        } else if baseline.scheduling.is_empty() && !candidate.scheduling.is_empty() {
            additive += 1;
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
            breaking += 1;
        }
    }

    if baseline.quality_gates != candidate.quality_gates {
        diagnostics.push(breaking_diag(
            "DPCS-COMPAT-026",
            "quality gates changed",
            "qualityGates",
        ));
        breaking += 1;
    }
    if baseline.failure_semantics != candidate.failure_semantics {
        diagnostics.push(breaking_diag(
            "DPCS-COMPAT-027",
            "failure semantics changed",
            "failureSemantics",
        ));
        breaking += 1;
    }
    if baseline.lineage != candidate.lineage {
        if baseline.lineage.is_some() && candidate.lineage.is_none() {
            diagnostics.push(breaking_diag(
                "DPCS-COMPAT-028",
                "lineage removed",
                "lineage",
            ));
            breaking += 1;
        } else if baseline.lineage.is_none() && candidate.lineage.is_some() {
            additive += 1;
        } else {
            diagnostics.push(breaking_diag(
                "DPCS-COMPAT-029",
                "lineage changed incompatibly",
                "lineage",
            ));
            breaking += 1;
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
        conditional += 1;
    }
    for key in candidate_ext.difference(&baseline_ext) {
        additive += 1;
        let _ = key;
    }

    diagnostics.sort_deterministic();
    let category = classify(breaking, additive, removed_optional, conditional);
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
    let mut breaking = 0usize;
    let mut additive = 0usize;
    let removed_optional = 0usize;
    let mut conditional = 0usize;

    if baseline.contract_id != candidate.contract_id {
        diagnostics.push(
            Diagnostic::compatibility_warning(
                "DPCS-COMPAT-040",
                categories::COMPATIBILITY,
                "plan contract ids differ",
            )
            .with_object_ref("contractId"),
        );
        conditional += 1;
    }

    let baseline_steps: BTreeSet<_> = baseline.steps.iter().map(|s| s.id.clone()).collect();
    let candidate_steps: BTreeSet<_> = candidate.steps.iter().map(|s| s.id.clone()).collect();
    for id in baseline_steps.difference(&candidate_steps) {
        diagnostics.push(breaking_diag(
            "DPCS-COMPAT-041",
            format!("plan step `{id}` removed"),
            format!("steps.{id}"),
        ));
        breaking += 1;
    }
    for id in candidate_steps.difference(&baseline_steps) {
        additive += 1;
        diagnostics.push(info_additive(
            "DPCS-COMPAT-042",
            format!("plan step `{id}` added"),
            format!("steps.{id}"),
        ));
        let _ = id;
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
    for edge in baseline_edges.difference(&candidate_edges) {
        diagnostics.push(breaking_diag(
            "DPCS-COMPAT-043",
            format!("dependency edge `{edge}` removed"),
            "dependencyEdges",
        ));
        breaking += 1;
    }
    for edge in candidate_edges.difference(&baseline_edges) {
        additive += 1;
        let _ = edge;
    }

    if baseline.step_order != candidate.step_order && breaking == 0 {
        diagnostics.push(
            Diagnostic::compatibility_warning(
                "DPCS-COMPAT-044",
                categories::COMPATIBILITY,
                "stepOrder differs while dependency edges remain compatible",
            )
            .with_object_ref("stepOrder"),
        );
        conditional += 1;
    }

    diagnostics.sort_deterministic();
    let category = classify(breaking, additive, removed_optional, conditional);
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

fn classify(
    breaking: usize,
    additive: usize,
    removed_optional: usize,
    conditional: usize,
) -> CompatibilityCategory {
    if breaking > 0 {
        CompatibilityCategory::Incompatible
    } else if conditional > 0 {
        CompatibilityCategory::ConditionallyCompatible
    } else if additive > 0 && removed_optional == 0 {
        CompatibilityCategory::BackwardCompatible
    } else if removed_optional > 0 && additive == 0 {
        CompatibilityCategory::ForwardCompatible
    } else if additive > 0 && removed_optional > 0 {
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
    breaking: &mut usize,
    additive: &mut usize,
    removed_optional: &mut usize,
) {
    let baseline_map: BTreeMap<_, _> = baseline.iter().map(|p| (p.id.clone(), p)).collect();
    let candidate_map: BTreeMap<_, _> = candidate.iter().map(|p| (p.id.clone(), p)).collect();

    for (id, port) in &baseline_map {
        match candidate_map.get(id) {
            None => {
                diagnostics.push(breaking_diag(
                    "DPCS-COMPAT-002",
                    format!("required interface port `{id}` removed"),
                    format!("{path}.{id}"),
                ));
                *breaking += 1;
                let _ = port;
            }
            Some(other) => {
                if port.contract_ref != other.contract_ref {
                    diagnostics.push(breaking_diag(
                        "DPCS-COMPAT-003",
                        format!("interface port `{id}` contractRef changed"),
                        format!("{path}.{id}.contractRef"),
                    ));
                    *breaking += 1;
                }
            }
        }
    }
    for id in candidate_map.keys() {
        if !baseline_map.contains_key(id) {
            *additive += 1;
            diagnostics.push(info_additive(
                "DPCS-COMPAT-004",
                format!("interface port `{id}` added"),
                format!("{path}.{id}"),
            ));
        }
    }
    let _ = removed_optional;
}

fn compare_steps(
    baseline: &[PipelineStep],
    candidate: &[PipelineStep],
    diagnostics: &mut ValidationReport,
    breaking: &mut usize,
    additive: &mut usize,
    removed_optional: &mut usize,
) {
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
                *breaking += 1;
            }
            Some(other) => {
                if step.step_type != other.step_type {
                    diagnostics.push(breaking_diag(
                        "DPCS-COMPAT-006",
                        format!("step `{id}` type changed"),
                        format!("steps.{id}.type"),
                    ));
                    *breaking += 1;
                }
                if step.contract_ref != other.contract_ref {
                    diagnostics.push(breaking_diag(
                        "DPCS-COMPAT-007",
                        format!("step `{id}` contractRef changed"),
                        format!("steps.{id}.contractRef"),
                    ));
                    *breaking += 1;
                }
            }
        }
    }
    for id in candidate_map.keys() {
        if !baseline_map.contains_key(id) {
            *additive += 1;
            diagnostics.push(info_additive(
                "DPCS-COMPAT-008",
                format!("step `{id}` added"),
                format!("steps.{id}"),
            ));
        }
    }
    let _ = removed_optional;
}

fn compare_string_pairs(
    path: &str,
    baseline: BTreeSet<String>,
    candidate: BTreeSet<String>,
    diagnostics: &mut ValidationReport,
    breaking: &mut usize,
    additive: &mut usize,
    removed_optional: &mut usize,
) {
    for edge in baseline.difference(&candidate) {
        diagnostics.push(breaking_diag(
            "DPCS-COMPAT-009",
            format!("`{path}` relation `{edge}` removed"),
            path,
        ));
        *breaking += 1;
    }
    for edge in candidate.difference(&baseline) {
        *additive += 1;
        let _ = edge;
    }
    let _ = removed_optional;
}

fn compare_refs(
    baseline: &PipelineContract,
    candidate: &PipelineContract,
    diagnostics: &mut ValidationReport,
    breaking: &mut usize,
    additive: &mut usize,
    removed_optional: &mut usize,
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
                *breaking += 1;
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
                    *breaking += 1;
                }
            }
        }
    }
    for id in candidate_map.keys() {
        if !baseline_map.contains_key(id) {
            *additive += 1;
        }
    }
    let _ = removed_optional;
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
        .map(|f| format!("{}=>{}", f.from, f.to))
        .collect()
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
