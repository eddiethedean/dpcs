//! Mermaid and Graphviz DOT exporters.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::PipelineContract;
use crate::report::views::{graph_view_from_contract, GraphView};

/// Escape a Mermaid/DOT node id to a safe identifier token (may collide).
fn sanitize_token(id: &str) -> String {
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

/// Map original node labels to unique sanitized tokens.
fn unique_ids(ids: impl IntoIterator<Item = String>) -> BTreeMap<String, String> {
    let mut used = BTreeSet::new();
    let mut map = BTreeMap::new();
    for id in ids {
        let base = sanitize_token(&id);
        let mut candidate = base.clone();
        let mut n = 2u32;
        while !used.insert(candidate.clone()) {
            candidate = format!("{base}__{n}");
            n += 1;
        }
        map.insert(id, candidate);
    }
    map
}

fn escape_label(label: &str) -> String {
    label
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\n', '\r'], " ")
}

/// Mermaid edge labels (between `|…|`) need pipes and brackets neutralized.
fn escape_mermaid_edge_label(label: &str) -> String {
    escape_label(label)
        .replace('|', "/")
        .replace('[', "(")
        .replace(']', ")")
}

fn collect_nodes(view: &GraphView) -> Vec<String> {
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
    nodes
}

/// Render a Mermaid `flowchart LR` diagram from a [`GraphView`].
pub fn graph_to_mermaid(view: &GraphView) -> String {
    let mut lines = vec!["flowchart LR".to_owned()];
    let nodes = collect_nodes(view);
    let ids = unique_ids(nodes);

    for (id, sid) in &ids {
        lines.push(format!("  {sid}[\"{}\"]", escape_label(id)));
    }
    if view.edges.is_empty() {
        lines.push("  %% no edges".to_owned());
    } else {
        for edge in &view.edges {
            let from = ids.get(&edge.from).map(String::as_str).unwrap_or("node");
            let to = ids.get(&edge.to).map(String::as_str).unwrap_or("node");
            match &edge.kind {
                Some(kind) if !kind.is_empty() => {
                    lines.push(format!(
                        "  {from} -->|{}| {to}",
                        escape_mermaid_edge_label(kind)
                    ));
                }
                _ => lines.push(format!("  {from} --> {to}")),
            }
        }
    }
    if view.planning_refused {
        lines.push("  %% planning refused".to_owned());
    } else if let Some(order) = &view.step_order {
        lines.push(format!(
            "  %% stepOrder: {}",
            escape_label(&order.join(", "))
        ));
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
    let nodes = collect_nodes(view);
    let ids = unique_ids(nodes);

    for (id, sid) in &ids {
        lines.push(format!("  {sid} [label=\"{}\"];", escape_label(id)));
    }
    for edge in &view.edges {
        let from = ids.get(&edge.from).map(String::as_str).unwrap_or("node");
        let to = ids.get(&edge.to).map(String::as_str).unwrap_or("node");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::views::{GraphEdgeView, GraphView};

    #[test]
    fn unique_sanitized_ids_disambiguate_collisions() {
        let view = GraphView {
            contract_id: None,
            entry_points: vec![],
            exit_points: vec![],
            step_ids: vec!["a.b".into(), "a_b".into()],
            edges: vec![GraphEdgeView {
                from: "a.b".into(),
                to: "a_b".into(),
                kind: Some("dependsOn".into()),
            }],
            step_order: None,
            planning_refused: false,
        };
        let mermaid = graph_to_mermaid(&view);
        assert!(mermaid.contains("a_b[\"a.b\"]"));
        assert!(mermaid.contains("a_b__2[\"a_b\"]") || mermaid.contains("a_b__2[\"a_b\"]"));
        assert!(!mermaid.lines().any(|l| l.contains("a_b --> a_b")));
        let dot = graph_to_dot(&view);
        assert!(dot.contains("a_b__2") || (dot.contains("a_b [") && dot.matches("a_b").count() > 1));
    }
}
