//! Markdown report renderers.

use crate::capabilities::CapabilityReport;
use crate::compatibility::CompatibilityReport;
use crate::diagnostics::{Diagnostic, DiagnosticReport, Severity, ValidationReport};
use crate::report::graph_export::graph_to_mermaid;
use crate::report::views::{GraphView, InspectView};

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Information => "information",
    }
}

fn render_diagnostic_list(out: &mut String, diagnostics: &[Diagnostic]) {
    if diagnostics.is_empty() {
        out.push_str("_No diagnostics._\n");
        return;
    }
    out.push_str("| Severity | Id | Stage | Object | Message |\n");
    out.push_str("| --- | --- | --- | --- | --- |\n");
    for d in diagnostics {
        let object = d
            .object_ref
            .as_deref()
            .unwrap_or("")
            .replace('|', "\\|")
            .replace('\n', " ");
        let message = d.message.replace('|', "\\|").replace('\n', " ");
        out.push_str(&format!(
            "| {} | `{}` | {} | {} | {} |\n",
            severity_label(d.severity),
            d.id.replace('|', "\\|"),
            d.stage.to_string().replace('|', "\\|"),
            object,
            message
        ));
    }
}

/// Render a validation report as Markdown.
pub fn validation_to_markdown(report: &ValidationReport) -> String {
    let mut out = String::from("# Validation Report\n\n");
    out.push_str(&format!(
        "**Summary:** {} error(s), {} warning(s), {} information(s)\n\n",
        report.error_count(),
        report.warning_count(),
        report.information_count()
    ));
    out.push_str("## Diagnostics\n\n");
    render_diagnostic_list(&mut out, &report.diagnostics);
    out
}

/// Render a diagnostic report wrapper as Markdown.
pub fn diagnostic_to_markdown(report: &DiagnosticReport) -> String {
    let mut out = String::from("# Diagnostic Report\n\n");
    out.push_str(&format!(
        "- **Processing result:** {}\n",
        report.processing_result
    ));
    if let Some(id) = &report.artifact_id {
        out.push_str(&format!("- **Artifact:** `{id}`\n"));
    }
    out.push_str(&format!(
        "- **Implementation:** {} {}\n\n",
        report.implementation.name, report.implementation.version
    ));
    out.push_str("## Diagnostics\n\n");
    render_diagnostic_list(&mut out, &report.diagnostics);
    out
}

/// Render an inspect view as Markdown.
pub fn inspect_to_markdown(view: &InspectView) -> String {
    let mut out = String::from("# Pipeline Inspect\n\n");
    out.push_str(&format!("- **Id:** `{}`\n", view.id));
    if let Some(name) = &view.name {
        out.push_str(&format!("- **Name:** {name}\n"));
    }
    out.push_str(&format!("- **Version:** {}\n", view.version));
    out.push_str(&format!("- **DPCS version:** {}\n", view.dpcs_version));
    out.push_str(&format!("- **Valid:** {}\n", view.valid));
    out.push_str(&format!("- **Steps:** {}\n", view.step_count));
    out.push_str(&format!("- **Edges:** {}\n", view.edge_count));
    out.push_str(&format!("- **Inputs:** {}\n", view.input_count));
    out.push_str(&format!("- **Outputs:** {}\n", view.output_count));
    out.push_str(&format!(
        "- **Contract references:** {}\n",
        view.contract_reference_count
    ));
    out.push_str(&format!("- **Data flow:** {}\n", view.data_flow_count));
    out.push_str(&format!(
        "- **Control flow:** {}\n",
        view.control_flow_count
    ));
    out.push_str(&format!("- **Scheduling:** {}\n", view.scheduling_count));
    out.push_str(&format!(
        "- **Quality gates:** {}\n",
        view.quality_gate_count
    ));
    out.push_str(&format!(
        "- **Failure semantics:** {}\n",
        view.failure_semantics_count
    ));
    out.push_str(&format!("- **Execution:** {}\n", view.has_execution));
    out.push_str(&format!("- **Lineage:** {}\n", view.has_lineage));
    if let Some(order) = &view.step_order {
        out.push_str(&format!("- **Step order:** {}\n", order.join(", ")));
    } else {
        out.push_str("- **Planning:** refused\n");
    }
    if !view.step_ids.is_empty() {
        out.push_str("\n## Steps\n\n");
        for id in &view.step_ids {
            out.push_str(&format!("- `{id}`\n"));
        }
    }
    out
}

/// Render a graph view as Markdown (with fenced Mermaid).
pub fn graph_to_markdown(view: &GraphView) -> String {
    let mut out = String::from("# Pipeline Graph\n\n");
    if let Some(id) = &view.contract_id {
        out.push_str(&format!("Contract: `{id}`\n\n"));
    }
    if !view.entry_points.is_empty() {
        out.push_str(&format!(
            "**Entry points:** {}\n\n",
            view.entry_points.join(", ")
        ));
    }
    if !view.exit_points.is_empty() {
        out.push_str(&format!(
            "**Exit points:** {}\n\n",
            view.exit_points.join(", ")
        ));
    }
    out.push_str("```mermaid\n");
    out.push_str(&graph_to_mermaid(view));
    out.push_str("\n```\n");
    if let Some(order) = &view.step_order {
        out.push_str(&format!("\n**Step order:** {}\n", order.join(", ")));
    } else if view.planning_refused {
        out.push_str("\n**Planning:** refused\n");
    }
    out
}

/// Render a capability report as Markdown.
pub fn capability_to_markdown(report: &CapabilityReport) -> String {
    let mut out = String::from("# Capability Report\n\n");
    out.push_str(&format!("- **Profile:** `{}`\n", report.profile_identity));
    if let Some(id) = &report.plan_contract_id {
        out.push_str(&format!("- **Contract:** `{id}`\n"));
    }
    out.push_str(&format!(
        "- **Satisfied:** {}\n",
        if report.satisfied.is_empty() {
            "_(none)_".to_owned()
        } else {
            report.satisfied.join(", ")
        }
    ));
    out.push_str(&format!(
        "- **Missing mandatory:** {}\n",
        if report.missing_mandatory.is_empty() {
            "_(none)_".to_owned()
        } else {
            report.missing_mandatory.join(", ")
        }
    ));
    out.push_str(&format!(
        "- **Unsupported optional:** {}\n\n",
        if report.unsupported_optional.is_empty() {
            "_(none)_".to_owned()
        } else {
            report.unsupported_optional.join(", ")
        }
    ));
    if !report.diagnostics.is_empty() {
        out.push_str("## Diagnostics\n\n");
        render_diagnostic_list(&mut out, &report.diagnostics);
    }
    out
}

/// Render a compatibility report as Markdown.
pub fn compatibility_to_markdown(report: &CompatibilityReport) -> String {
    let mut out = String::from("# Compatibility Report\n\n");
    out.push_str(&format!(
        "- **Baseline:** `{}@{}`\n",
        report.baseline_id, report.baseline_version
    ));
    out.push_str(&format!(
        "- **Candidate:** `{}@{}`\n",
        report.candidate_id, report.candidate_version
    ));
    out.push_str(&format!("- **Category:** {}\n\n", report.category));
    out.push_str("## Diagnostics\n\n");
    render_diagnostic_list(&mut out, &report.diagnostics);
    out
}
