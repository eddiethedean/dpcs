//! CLI output helpers: `--format`, `--out`, colored text.

use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use crate::capabilities::CapabilityReport;
use crate::compatibility::CompatibilityReport;
use crate::diagnostics::{DiagnosticReport, Severity, ValidationReport};
use crate::error::{Error, Result};
use crate::report::{
    capability_to_html, capability_to_markdown, compatibility_to_html, compatibility_to_markdown,
    diagnostic_to_html, diagnostic_to_markdown, graph_to_dot, graph_to_html, graph_to_markdown,
    graph_to_mermaid, inspect_to_html, inspect_to_markdown, validation_to_html,
    validation_to_markdown, GraphView, InspectView, ReportFormat,
};

/// Emission options derived from `--json` / `--format` / `--out`.
#[derive(Debug, Clone)]
pub struct EmitOpts {
    /// Selected output format.
    pub format: ReportFormat,
    /// Optional destination path (stdout when `None`).
    pub out: Option<PathBuf>,
}

impl EmitOpts {
    /// Build options from CLI flags. `--json` aliases `--format json` when format is unset.
    pub fn from_flags(json: bool, format: Option<&str>, out: Option<PathBuf>) -> Result<Self> {
        let format = if let Some(raw) = format {
            ReportFormat::parse(raw)?
        } else if json {
            ReportFormat::Json
        } else {
            ReportFormat::Text
        };
        Ok(Self { format, out })
    }

    pub fn require_formats(&self, allowed: &[ReportFormat]) -> Result<()> {
        if allowed.contains(&self.format) {
            Ok(())
        } else {
            Err(Error::Serialization(format!(
                "format `{}` is not supported here (allowed: {})",
                self.format,
                allowed
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join("|")
            )))
        }
    }

    /// Remap graph-only formats so diagnostic emission stays valid.
    pub fn for_diagnostics(&self) -> Self {
        let format = match self.format {
            ReportFormat::Mermaid | ReportFormat::Dot => ReportFormat::Text,
            other => other,
        };
        Self {
            format,
            out: self.out.clone(),
        }
    }
}

/// Write a string body to `--out` or stdout.
pub fn emit_body(opts: &EmitOpts, body: impl AsRef<str>) -> Result<()> {
    let body = body.as_ref();
    if let Some(path) = &opts.out {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|err| Error::Io {
                    path: parent.to_path_buf(),
                    source: err,
                })?;
            }
        }
        std::fs::write(path, body.as_bytes()).map_err(|err| Error::Io {
            path: path.clone(),
            source: err,
        })?;
    } else {
        let mut stdout = io::stdout().lock();
        stdout.write_all(body.as_bytes()).map_err(|err| {
            Error::Serialization(format!("failed writing report to stdout: {err}"))
        })?;
        if !body.ends_with('\n') {
            stdout.write_all(b"\n").map_err(|err| {
                Error::Serialization(format!("failed writing report to stdout: {err}"))
            })?;
        }
    }
    Ok(())
}

fn color_enabled(opts: &EmitOpts) -> bool {
    opts.out.is_none() && io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

fn paint_severity(opts: &EmitOpts, severity: Severity) -> String {
    let label = match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Information => "information",
    };
    if !color_enabled(opts) {
        return label.to_owned();
    }
    match severity {
        Severity::Error => format!("\u{1b}[31m{label}\u{1b}[0m"),
        Severity::Warning => format!("\u{1b}[33m{label}\u{1b}[0m"),
        Severity::Information => format!("\u{1b}[34m{label}\u{1b}[0m"),
    }
}

/// Emit a validation report in the selected format.
pub fn emit_validation(opts: &EmitOpts, report: &ValidationReport) -> Result<()> {
    opts.require_formats(&[
        ReportFormat::Text,
        ReportFormat::Json,
        ReportFormat::Markdown,
        ReportFormat::Html,
    ])?;
    match opts.format {
        ReportFormat::Text => emit_body(opts, format_validation_text(opts, report)),
        ReportFormat::Json => {
            let payload = serde_json::to_string_pretty(report).map_err(|err| {
                Error::Serialization(format!("failed to serialize diagnostics: {err}"))
            })?;
            emit_body(opts, payload)
        }
        ReportFormat::Markdown => emit_body(opts, validation_to_markdown(report)),
        ReportFormat::Html => emit_body(opts, validation_to_html(report)),
        other => Err(Error::Serialization(format!(
            "format `{other}` not supported for validation reports"
        ))),
    }
}

/// Emit a diagnostic report (`validate` / `diagnostics` JSON path) in the selected format.
pub fn emit_diagnostic_report(opts: &EmitOpts, report: &DiagnosticReport) -> Result<()> {
    opts.require_formats(&[
        ReportFormat::Text,
        ReportFormat::Json,
        ReportFormat::Markdown,
        ReportFormat::Html,
    ])?;
    match opts.format {
        ReportFormat::Text => {
            let mut tmp = ValidationReport::new();
            tmp.diagnostics = report.diagnostics.clone();
            emit_body(opts, format_validation_text(opts, &tmp))
        }
        ReportFormat::Json => {
            let payload = serde_json::to_string_pretty(report).map_err(|err| {
                Error::Serialization(format!("failed to serialize diagnostic report: {err}"))
            })?;
            emit_body(opts, payload)
        }
        ReportFormat::Markdown => emit_body(opts, diagnostic_to_markdown(report)),
        ReportFormat::Html => emit_body(opts, diagnostic_to_html(report)),
        other => Err(Error::Serialization(format!(
            "format `{other}` not supported for diagnostic reports"
        ))),
    }
}

fn format_validation_text(opts: &EmitOpts, report: &ValidationReport) -> String {
    if report.diagnostics.is_empty() {
        return "valid: no diagnostics".to_owned();
    }
    let mut out = String::new();
    for diagnostic in &report.diagnostics {
        let object = diagnostic
            .object_ref
            .as_deref()
            .map(|value| format!(" @ {value}"))
            .unwrap_or_default();
        let location = diagnostic
            .source_location
            .as_deref()
            .map(|value| format!(" ({value})"))
            .unwrap_or_default();
        out.push_str(&format!(
            "{} {}: {}{} — {}{}\n",
            paint_severity(opts, diagnostic.severity),
            diagnostic.id,
            diagnostic.stage,
            object,
            diagnostic.message,
            location
        ));
        if let Some(remediation) = &diagnostic.remediation {
            out.push_str(&format!("  remediation: {remediation}\n"));
        }
    }
    out.push_str(&format!(
        "summary: {} error(s), {} warning(s), {} information(s)",
        report.error_count(),
        report.warning_count(),
        report.information_count()
    ));
    out
}

/// Emit an inspect view.
pub fn emit_inspect(opts: &EmitOpts, view: &InspectView) -> Result<()> {
    opts.require_formats(&[
        ReportFormat::Text,
        ReportFormat::Json,
        ReportFormat::Markdown,
        ReportFormat::Html,
    ])?;
    match opts.format {
        ReportFormat::Text => emit_body(opts, format_inspect_text(view)),
        ReportFormat::Json => {
            let payload = serde_json::to_string_pretty(view).map_err(|err| {
                Error::Serialization(format!("failed to serialize inspect summary: {err}"))
            })?;
            emit_body(opts, payload)
        }
        ReportFormat::Markdown => emit_body(opts, inspect_to_markdown(view)),
        ReportFormat::Html => emit_body(opts, inspect_to_html(view)),
        other => Err(Error::Serialization(format!(
            "format `{other}` not supported for inspect"
        ))),
    }
}

fn format_inspect_text(view: &InspectView) -> String {
    let mut lines = vec![
        format!("id: {}", view.id),
        format!("version: {}", view.version),
        format!("dpcsVersion: {}", view.dpcs_version),
        format!("steps: {}", view.step_count),
        format!("edges: {}", view.edge_count),
        format!("inputs: {}", view.input_count),
        format!("outputs: {}", view.output_count),
        format!("contractReferences: {}", view.contract_reference_count),
        format!("dataFlow: {}", view.data_flow_count),
        format!("controlFlow: {}", view.control_flow_count),
        format!("scheduling: {}", view.scheduling_count),
        format!("qualityGates: {}", view.quality_gate_count),
        format!("failureSemantics: {}", view.failure_semantics_count),
        format!("execution: {}", view.has_execution),
        format!("lineage: {}", view.has_lineage),
        format!("valid: {}", view.valid),
    ];
    if let Some(name) = &view.name {
        lines.insert(1, format!("name: {name}"));
    }
    if let Some(order) = &view.step_order {
        lines.push(format!("stepOrder: {}", order.join(", ")));
    } else {
        lines.push("planning: refused".to_owned());
    }
    lines.join("\n")
}

/// Emit a graph view.
pub fn emit_graph(opts: &EmitOpts, view: &GraphView) -> Result<()> {
    opts.require_formats(&[
        ReportFormat::Text,
        ReportFormat::Json,
        ReportFormat::Markdown,
        ReportFormat::Html,
        ReportFormat::Mermaid,
        ReportFormat::Dot,
    ])?;
    match opts.format {
        ReportFormat::Text => emit_body(opts, format_graph_text(view)),
        ReportFormat::Json => {
            let payload = serde_json::to_string_pretty(view).map_err(|err| {
                Error::Serialization(format!("failed to serialize graph payload: {err}"))
            })?;
            emit_body(opts, payload)
        }
        ReportFormat::Markdown => emit_body(opts, graph_to_markdown(view)),
        ReportFormat::Html => emit_body(opts, graph_to_html(view)),
        ReportFormat::Mermaid => emit_body(opts, graph_to_mermaid(view)),
        ReportFormat::Dot => emit_body(opts, graph_to_dot(view)),
    }
}

fn format_graph_text(view: &GraphView) -> String {
    let mut lines = Vec::new();
    if !view.entry_points.is_empty() {
        lines.push(format!("entryPoints: {}", view.entry_points.join(", ")));
    }
    if !view.exit_points.is_empty() {
        lines.push(format!("exitPoints: {}", view.exit_points.join(", ")));
    }
    if view.edges.is_empty() {
        lines.push("(no edges)".to_owned());
    } else {
        for edge in &view.edges {
            match &edge.kind {
                Some(kind) => lines.push(format!("{} -> {} ({kind})", edge.from, edge.to)),
                None => lines.push(format!("{} -> {}", edge.from, edge.to)),
            }
        }
    }
    match &view.step_order {
        Some(order) => lines.push(format!("stepOrder: {}", order.join(", "))),
        None if view.planning_refused => lines.push("planning: refused".to_owned()),
        None => {}
    }
    lines.join("\n")
}

/// Emit a capability report.
pub fn emit_capability(opts: &EmitOpts, report: &CapabilityReport, match_ok: bool) -> Result<()> {
    opts.require_formats(&[
        ReportFormat::Text,
        ReportFormat::Json,
        ReportFormat::Markdown,
        ReportFormat::Html,
    ])?;
    match opts.format {
        ReportFormat::Text => emit_body(opts, format_capability_text(opts, report, match_ok)),
        ReportFormat::Json => {
            let payload = serde_json::to_string_pretty(report).map_err(|err| {
                Error::Serialization(format!("serialize capability report: {err}"))
            })?;
            emit_body(opts, payload)
        }
        ReportFormat::Markdown => emit_body(opts, capability_to_markdown(report)),
        ReportFormat::Html => emit_body(opts, capability_to_html(report)),
        other => Err(Error::Serialization(format!(
            "format `{other}` not supported for capabilities"
        ))),
    }
}

fn format_capability_text(opts: &EmitOpts, report: &CapabilityReport, match_ok: bool) -> String {
    let mut lines = vec![format!("profile: {}", report.profile_identity)];
    if let Some(contract_id) = &report.plan_contract_id {
        lines.push(format!("contractId: {contract_id}"));
    }
    lines.push(format!("satisfied: {}", report.satisfied.join(", ")));
    if !report.unsupported_optional.is_empty() {
        lines.push(format!(
            "unsupportedOptional: {}",
            report.unsupported_optional.join(", ")
        ));
    }
    if !report.missing_mandatory.is_empty() {
        lines.push(format!(
            "missingMandatory: {}",
            report.missing_mandatory.join(", ")
        ));
    }
    for diagnostic in &report.diagnostics {
        lines.push(format!(
            "{} {}: {} — {}",
            paint_severity(opts, diagnostic.severity),
            diagnostic.id,
            diagnostic.stage,
            diagnostic.message
        ));
    }
    if match_ok {
        lines.push("match: ok".to_owned());
    }
    lines.join("\n")
}

/// Emit a compatibility report.
pub fn emit_compatibility(opts: &EmitOpts, report: &CompatibilityReport) -> Result<()> {
    opts.require_formats(&[
        ReportFormat::Text,
        ReportFormat::Json,
        ReportFormat::Markdown,
        ReportFormat::Html,
    ])?;
    match opts.format {
        ReportFormat::Text => emit_body(opts, format_compatibility_text(opts, report)),
        ReportFormat::Json => {
            let payload = serde_json::to_string_pretty(report).map_err(|err| {
                Error::Serialization(format!("serialize compatibility report: {err}"))
            })?;
            emit_body(opts, payload)
        }
        ReportFormat::Markdown => emit_body(opts, compatibility_to_markdown(report)),
        ReportFormat::Html => emit_body(opts, compatibility_to_html(report)),
        other => Err(Error::Serialization(format!(
            "format `{other}` not supported for compatibility"
        ))),
    }
}

fn format_compatibility_text(opts: &EmitOpts, report: &CompatibilityReport) -> String {
    let mut lines = vec![
        format!(
            "baseline: {}@{}",
            report.baseline_id, report.baseline_version
        ),
        format!(
            "candidate: {}@{}",
            report.candidate_id, report.candidate_version
        ),
        format!("category: {}", report.category),
    ];
    for diagnostic in &report.diagnostics {
        lines.push(format!(
            "{} {}: {} — {}",
            paint_severity(opts, diagnostic.severity),
            diagnostic.id,
            diagnostic.stage,
            diagnostic.message
        ));
    }
    lines.push(format!(
        "compatibility: {}",
        if report.category.is_compatible() {
            "ok"
        } else {
            "incompatible"
        }
    ));
    lines.join("\n")
}
