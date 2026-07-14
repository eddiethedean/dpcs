//! Presentation views and report rendering (Markdown, HTML, Mermaid, DOT).

mod format;
mod graph_export;
mod html;
mod markdown;
mod views;

pub use format::ReportFormat;
pub use graph_export::{graph_to_dot, graph_to_mermaid, to_dot, to_mermaid};
pub use html::{
    capability_to_html, compatibility_to_html, diagnostic_to_html, graph_to_html, inspect_to_html,
    validation_to_html,
};
pub use markdown::{
    capability_to_markdown, compatibility_to_markdown, diagnostic_to_markdown, graph_to_markdown,
    inspect_to_markdown, validation_to_markdown,
};
pub use views::{
    graph_view_from_contract, inspect_view_from_contract, GraphEdgeView, GraphView, InspectView,
};
