//! Render a context graph's area dependencies as a Mermaid `graph`.
//!
//! Nodes are the project's areas; edges are the resolved import references
//! between them, labelled with how many references cross. The output is a
//! self-contained Mermaid diagram that renders on GitHub, in Blink's website,
//! or any Mermaid viewer.

use std::collections::BTreeMap;
use std::fmt::Write;

use blink_context::ContextGraph;

pub fn render(graph: &ContextGraph) -> String {
    let mut out = String::new();
    out.push_str("graph TD\n");

    // Stable node ids for each area.
    let mut ids: BTreeMap<&str, String> = BTreeMap::new();
    for (i, area) in graph.areas.iter().enumerate() {
        ids.insert(area.path.as_str(), format!("n{i}"));
    }

    if graph.areas.is_empty() {
        out.push_str("  empty[\"(no areas)\"]\n");
        return out;
    }

    // Declare nodes, labelled with the area and its symbol count.
    for area in &graph.areas {
        let id = &ids[area.path.as_str()];
        let _ = writeln!(
            out,
            "  {id}[\"{}<br/>{} files · {} symbols\"]",
            escape(&area.path),
            area.files,
            area.symbols
        );
    }

    // Edges between areas.
    for edge in graph.area_edges() {
        let (Some(from), Some(to)) = (ids.get(edge.from.as_str()), ids.get(edge.to.as_str()))
        else {
            continue;
        };
        let _ = writeln!(out, "  {from} -->|{}| {to}", edge.count);
    }

    out
}

/// Escape characters that would break a Mermaid quoted label.
fn escape(s: &str) -> String {
    s.replace('"', "&quot;")
}
