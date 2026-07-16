//! Serialize a [`ContextGraph`] to a chosen format.
//!
//! Four formats, all rendered from the same graph so they never disagree:
//! - **JSON** — the graph verbatim, for tooling.
//! - **YAML** — the same data, human-diffable.
//! - **Markdown** — a readable project document (overview, areas, dependencies,
//!   commands, key relationships).
//! - **Graph** — a Mermaid `graph` of how areas depend on one another.
//!
//! Nothing is added that isn't in the graph; export is a pure re-encoding.

use std::fmt;
use std::str::FromStr;

use blink_context::ContextGraph;

mod markdown;
mod mermaid;
mod yaml;

/// A supported export format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Yaml,
    Markdown,
    /// A Mermaid architecture graph.
    Graph,
}

impl ExportFormat {
    /// The conventional output filename for this format.
    pub fn default_filename(self) -> &'static str {
        match self {
            ExportFormat::Json => "blink-context.json",
            ExportFormat::Yaml => "blink-context.yaml",
            ExportFormat::Markdown => "blink-context.md",
            ExportFormat::Graph => "blink-context.mmd",
        }
    }

    /// Every accepted format name, for help/error text.
    pub fn variants() -> &'static [&'static str] {
        &["json", "yaml", "markdown", "graph"]
    }
}

impl fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ExportFormat::Json => "json",
            ExportFormat::Yaml => "yaml",
            ExportFormat::Markdown => "markdown",
            ExportFormat::Graph => "graph",
        };
        f.write_str(s)
    }
}

impl FromStr for ExportFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "yaml" | "yml" => Ok(ExportFormat::Yaml),
            "markdown" | "md" => Ok(ExportFormat::Markdown),
            "graph" | "mermaid" | "mmd" => Ok(ExportFormat::Graph),
            other => Err(format!(
                "unknown export format '{other}' (expected one of: {})",
                ExportFormat::variants().join(", ")
            )),
        }
    }
}

/// Render `graph` in `format`.
pub fn export(graph: &ContextGraph, format: ExportFormat) -> String {
    match format {
        ExportFormat::Json => {
            // Infallible in practice (the graph is plain data); fall back to a
            // valid empty object rather than panicking.
            serde_json::to_string_pretty(graph).unwrap_or_else(|_| "{}".to_string())
        }
        ExportFormat::Yaml => {
            let value = serde_json::to_value(graph).unwrap_or(serde_json::Value::Null);
            yaml::to_yaml(&value)
        }
        ExportFormat::Markdown => markdown::render(graph),
        ExportFormat::Graph => mermaid::render(graph),
    }
}

#[cfg(test)]
mod tests;
