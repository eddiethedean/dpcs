//! Mermaid and Graphviz DOT exporters.

use crate::model::PipelineContract;
use crate::report::views::{graph_view_from_contract, GraphView};

/// Escape a Mermaid/DOT node id to a safe identifier token.
fn sanitize_id(id: &str) -> String {
    let mut out = String::with_capacity(id.len());
    for c in id.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push_str("node");
    } else if out.as_bytes()[0].is_ascii_digit() {
        out.insert(0, 'n');
    }
    out
}

fn escape_label(label: &str) -> String {
    label.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Render a Mermaid `flowchart LR` diagram from a [`GraphView`].
pub fn graph_to_mermaid(view: &GraphView) -> String {
    let mut lines = vec!["flowchart LR".to_owned()];
    let mut nodes: Vec<String> = view.step_ids.clone();
    for edge in &view.edges {
        if !nodes.iter().any(|n| n == &edge.from) {
            nodes.push(edge.from.clone());
        }
        if !nodes.iter().any(|n| n == &edge.to) {
            nodes.push(edge.to.clone());
        }
    }
    for ep in view
        .entry_points
        .iter()
        .chain(view.exit_points.iter())
    {
        if !nodes.iter().any(|n| n == ep) {
            nodes.push(ep.clone());
        }
    }

    for id in &nodes {
        let sid = sanitize_id(id);
        lines.push(format!("  {sid}[\"{}\"]", escape_label(id)));
    }
    if view.edges.is_empty() {
        lines.push("  %% no edges".to_owned());
    } else {
        for edge in &view.edges {
            let from = sanitize_id(&edge.from);
            let to = sanitize_id(&edge.to);
            match &edge.kind {
                Some(kind) if !kind.is_empty() => {
                    lines.push(format!("  {from} -->|{}| {to}", escape_label(kind)));
                }
                _ => lines.push(format!("  {from} --> {to}")),
            }
        }
    }
    if view.planning_refused {
        lines.push("  %% planning refused".to_owned());
    } else if let Some(order) = &view.step_order {
        lines.push(format!("  %% stepOrder: {}", order.join(", ")));
    }
    lines.join("\n")
}

/// Render Graphviz DOT from a [`GraphView`].
pub fn graph_to_dot(view: &GraphView) -> String {
    let mut lines = vec![
        "digraph dpcs {".to_owned(),
        "  rankdir=LR;".to_owned(),
        "  node [shape=box];".to_owned(),
    ];
    let mut nodes: Vec<String> = view.step_ids.clone();
    for edge in &view.edges {
        if !nodes.iter().any(|n| n == &edge.from) {
            nodes.push(edge.from.clone());
        }
        if !nodes.iter().any(|n| n == &edge.to) {
            nodes.push(edge.to.clone());
        }
    }
    for id in &nodes {
        let sid = sanitize_id(id);
        lines.push(format!(
            "  {sid} [label=\"{}\"];",
            escape_label(id)
        ));
    }
    for edge in &view.edges {
        let from = sanitize_id(&edge.from);
        let to = sanitize_id(&edge.to);
        match &edge.kind {
            Some(kind) if !kind.is_empty() => {
                lines.push(format!(
                    "  {from} -> {to} [label=\"{}\"];",
                    escape_label(kind)
                ));
            }
            _ => lines.push(format!("  {from} -> {to};")),
        }
    }
    if view.planning_refused {
        lines.push("  // planning refused".to_owned());
    }
    lines.push("}".to_owned());
    lines.join("\n")
}

/// Mermaid export for a pipeline contract.
pub fn to_mermaid(contract: &PipelineContract) -> String {
    graph_to_mermaid(&graph_view_from_contract(contract))
}

/// DOT export for a pipeline contract.
pub fn to_dot(contract: &PipelineContract) -> String {
    graph_to_dot(&graph_view_from_contract(contract))
}
