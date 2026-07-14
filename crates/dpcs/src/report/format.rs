//! Report output format identifiers.

use std::str::FromStr;

use crate::error::{Error, Result};

/// Supported report / export formats for CLI and library renderers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReportFormat {
    /// Human-oriented plain or richly styled terminal text.
    #[default]
    Text,
    /// Pretty-printed JSON.
    Json,
    /// Markdown document.
    Markdown,
    /// Self-contained HTML document.
    Html,
    /// Mermaid flowchart source.
    Mermaid,
    /// Graphviz DOT source.
    Dot,
}

impl ReportFormat {
    /// Parse from a CLI flag value (`text`, `json`, `markdown`, `html`, `mermaid`, `dot`).
    pub fn parse(value: &str) -> Result<Self> {
        value.parse()
    }

    /// Returns `true` when this format is a graph export.
    pub fn is_graph_export(self) -> bool {
        matches!(self, Self::Mermaid | Self::Dot)
    }
}

impl FromStr for ReportFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "text" | "plain" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            "markdown" | "md" => Ok(Self::Markdown),
            "html" => Ok(Self::Html),
            "mermaid" => Ok(Self::Mermaid),
            "dot" | "graphviz" => Ok(Self::Dot),
            other => Err(Error::Serialization(format!(
                "unknown report format `{other}` (expected text|json|markdown|html|mermaid|dot)"
            ))),
        }
    }
}

impl std::fmt::Display for ReportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Text => "text",
            Self::Json => "json",
            Self::Markdown => "markdown",
            Self::Html => "html",
            Self::Mermaid => "mermaid",
            Self::Dot => "dot",
        })
    }
}
