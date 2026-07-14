//! Self-contained HTML report renderers (diagnostics/inspect are fully offline;
//! graph pages embed Mermaid source and optionally load a CDN renderer).

use crate::capabilities::CapabilityReport;
use crate::compatibility::CompatibilityReport;
use crate::diagnostics::{Diagnostic, DiagnosticReport, Severity, ValidationReport};
use crate::report::graph_export::graph_to_mermaid;
use crate::report::views::{GraphView, InspectView};

fn esc(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn wrap_document(title: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>{title}</title>
<style>
body {{ font-family: ui-sans-serif, system-ui, sans-serif; margin: 2rem; color: #1a1a1a; background: #fafafa; }}
h1 {{ font-size: 1.5rem; }}
table {{ border-collapse: collapse; width: 100%; margin: 1rem 0; background: #fff; }}
th, td {{ border: 1px solid #ddd; padding: 0.4rem 0.6rem; text-align: left; vertical-align: top; }}
th {{ background: #f0f0f0; }}
code {{ background: #eee; padding: 0.1rem 0.25rem; border-radius: 3px; }}
.error {{ color: #b00020; }}
.warning {{ color: #9a6700; }}
.info {{ color: #0550ae; }}
.meta {{ color: #555; }}
pre {{ background: #fff; border: 1px solid #ddd; padding: 1rem; overflow: auto; }}
</style>
</head>
<body>
{body}
</body>
</html>
"#,
        title = esc(title),
        body = body
    )
}

fn severity_class(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Information => "info",
    }
}

fn diagnostics_table(diagnostics: &[Diagnostic]) -> String {
    if diagnostics.is_empty() {
        return "<p><em>No diagnostics.</em></p>".to_owned();
    }
    let mut out = String::from(
        "<table><thead><tr><th>Severity</th><th>Id</th><th>Stage</th><th>Object</th><th>Message</th></tr></thead><tbody>",
    );
    for d in diagnostics {
        out.push_str(&format!(
            "<tr><td class=\"{cls}\">{sev}</td><td><code>{id}</code></td><td>{stage}</td><td>{object}</td><td>{message}</td></tr>",
            cls = severity_class(d.severity),
            sev = esc(&d.severity.to_string()),
            id = esc(&d.id),
            stage = esc(&d.stage.to_string()),
            object = esc(d.object_ref.as_deref().unwrap_or("")),
            message = esc(&d.message),
        ));
    }
    out.push_str("</tbody></table>");
    out
}

/// HTML for a validation report.
pub fn validation_to_html(report: &ValidationReport) -> String {
    let body = format!(
        "<h1>Validation Report</h1><p class=\"meta\">{} error(s), {} warning(s), {} information(s)</p>{}",
        report.error_count(),
        report.warning_count(),
        report.information_count(),
        diagnostics_table(&report.diagnostics)
    );
    wrap_document("Validation Report", &body)
}

/// HTML for a diagnostic report.
pub fn diagnostic_to_html(report: &DiagnosticReport) -> String {
    let mut body = format!(
        "<h1>Diagnostic Report</h1><ul><li>Processing result: <code>{}</code></li>",
        esc(&report.processing_result.to_string())
    );
    if let Some(id) = &report.artifact_id {
        body.push_str(&format!("<li>Artifact: <code>{}</code></li>", esc(id)));
    }
    body.push_str(&format!(
        "<li>Implementation: {} {}</li></ul>{}",
        esc(&report.implementation.name),
        esc(&report.implementation.version),
        diagnostics_table(&report.diagnostics)
    ));
    wrap_document("Diagnostic Report", &body)
}

/// HTML for an inspect view.
pub fn inspect_to_html(view: &InspectView) -> String {
    let mut body = format!(
        "<h1>Pipeline Inspect</h1><ul>\
<li>Id: <code>{}</code></li>\
<li>Version: {}</li>\
<li>DPCS version: {}</li>\
<li>Valid: {}</li>\
<li>Steps: {}</li>\
<li>Edges: {}</li>\
<li>Inputs: {} / Outputs: {}</li>",
        esc(&view.id),
        esc(&view.version),
        esc(&view.dpcs_version),
        view.valid,
        view.step_count,
        view.edge_count,
        view.input_count,
        view.output_count
    );
    if let Some(name) = &view.name {
        body.push_str(&format!("<li>Name: {}</li>", esc(name)));
    }
    if let Some(order) = &view.step_order {
        body.push_str(&format!("<li>Step order: {}</li>", esc(&order.join(", "))));
    } else {
        body.push_str("<li>Planning: refused</li>");
    }
    body.push_str("</ul>");
    if !view.step_ids.is_empty() {
        body.push_str("<h2>Steps</h2><ul>");
        for id in &view.step_ids {
            body.push_str(&format!("<li><code>{}</code></li>", esc(id)));
        }
        body.push_str("</ul>");
    }
    wrap_document("Pipeline Inspect", &body)
}

/// HTML for a graph view (Mermaid source always embedded; CDN render is optional).
pub fn graph_to_html(view: &GraphView) -> String {
    let mermaid = graph_to_mermaid(view);
    let title = view
        .contract_id
        .as_deref()
        .map(|id| format!("Graph — {id}"))
        .unwrap_or_else(|| "Pipeline Graph".to_owned());
    let body = format!(
        "<h1>{}</h1>\
<p class=\"meta\">Mermaid source is embedded for offline use. Rendering below requires network access to the Mermaid CDN.</p>\
<h2>Mermaid source</h2>\
<pre><code>{}</code></pre>\
<h2>Rendered graph</h2>\
<div class=\"mermaid\">\n{}\n</div>\
<script type=\"module\">\
import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs';\
mermaid.initialize({{ startOnLoad: true }});\
</script>",
        esc(&title),
        esc(&mermaid),
        esc(&mermaid)
    );
    wrap_document(&title, &body)
}

/// HTML for a capability report.
pub fn capability_to_html(report: &CapabilityReport) -> String {
    let mut body = format!(
        "<h1>Capability Report</h1><ul>\
<li>Profile: <code>{}</code></li>",
        esc(&report.profile_identity),
    );
    if let Some(id) = &report.plan_contract_id {
        body.push_str(&format!("<li>Contract: <code>{}</code></li>", esc(id)));
    }
    body.push_str(&format!(
        "<li>Satisfied: {}</li>\
<li>Missing mandatory: {}</li>\
<li>Unsupported optional: {}</li></ul>{}",
        esc(&report.satisfied.join(", ")),
        esc(&report.missing_mandatory.join(", ")),
        esc(&report.unsupported_optional.join(", ")),
        diagnostics_table(&report.diagnostics)
    ));
    wrap_document("Capability Report", &body)
}

/// HTML for a compatibility report.
pub fn compatibility_to_html(report: &CompatibilityReport) -> String {
    let body = format!(
        "<h1>Compatibility Report</h1><ul>\
<li>Baseline: <code>{}@{}</code></li>\
<li>Candidate: <code>{}@{}</code></li>\
<li>Category: <code>{}</code></li></ul>{}",
        esc(&report.baseline_id),
        esc(&report.baseline_version),
        esc(&report.candidate_id),
        esc(&report.candidate_version),
        esc(&report.category.to_string()),
        diagnostics_table(&report.diagnostics)
    );
    wrap_document("Compatibility Report", &body)
}
