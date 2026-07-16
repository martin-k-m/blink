//! Deterministic, single-file explanation.
//!
//! `blink explain <file>` must never invent prose about what a file "does".
//! Everything here is a real signal read from the file or the graph: the file's
//! own leading doc comment (verbatim), the symbols it declares, the project
//! files it imports, the external packages it imports, and the project files
//! that import it. No language model, no guessed "responsibilities".

use std::path::Path;

use blink_index::FileLang;

use crate::imports;
use crate::model::{ContextGraph, SymbolRef};

/// A file's explanation, assembled entirely from measured facts.
#[derive(Debug, Clone)]
pub struct FileExplanation {
    pub path: String,
    pub area: String,
    pub lang: Option<String>,
    pub lines: usize,
    pub size_bytes: u64,
    /// The file's leading doc comment, verbatim (module doc, JSDoc, docstring),
    /// or `None` if it has none.
    pub doc: Option<String>,
    /// Top-level symbols declared in this file.
    pub symbols: Vec<SymbolRef>,
    /// Project files this file references.
    pub references: Vec<String>,
    /// External packages/modules this file imports.
    pub external: Vec<String>,
    /// Project files that reference this file.
    pub dependents: Vec<String>,
}

impl ContextGraph {
    /// Explain a single project-relative file, or `None` if it isn't in the
    /// graph.
    pub fn explain(&self, path: &str) -> Option<FileExplanation> {
        let node = self.file(path)?;
        let text = std::fs::read_to_string(self.root.join(path)).unwrap_or_default();
        let lang = FileLang::from_path(Path::new(path));

        let doc = lang.and_then(|l| leading_doc(l, &text));
        let external = match lang {
            Some(l) if !text.is_empty() => {
                let paths = self.files.iter().map(|f| f.path.clone()).collect();
                imports::scan(l, &text, path, &paths).external
            }
            _ => Vec::new(),
        };

        Some(FileExplanation {
            path: node.path.clone(),
            area: node.area.clone(),
            lang: node.lang.clone(),
            lines: node.lines,
            size_bytes: node.size_bytes,
            doc,
            symbols: node.symbols.clone(),
            references: self
                .dependencies_of(path)
                .into_iter()
                .map(String::from)
                .collect(),
            external,
            dependents: self
                .dependents_of(path)
                .into_iter()
                .map(String::from)
                .collect(),
        })
    }
}

/// Extract the leading documentation comment from a file, verbatim.
fn leading_doc(lang: FileLang, text: &str) -> Option<String> {
    let doc = match lang {
        FileLang::Rust => line_doc(text, &["//!", "///"]),
        FileLang::Python => python_docstring(text),
        FileLang::TypeScript | FileLang::JavaScript | FileLang::Go => block_or_line_doc(text),
    };
    doc.filter(|d| !d.trim().is_empty())
        .map(|d| d.trim().to_string())
}

/// Collect the leading run of line-comment doc lines matching one of `markers`.
fn line_doc(text: &str, markers: &[&str]) -> Option<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let t = line.trim_start();
        if t.is_empty() && out.is_empty() {
            continue; // allow blank lines before the doc block
        }
        match markers.iter().find_map(|m| t.strip_prefix(*m)) {
            Some(rest) => out.push(rest.trim_start().to_string()),
            None => break,
        }
    }
    (!out.is_empty()).then(|| out.join("\n"))
}

/// A leading `/** ... */` / `/* ... */` block comment, or failing that a
/// leading run of `//` line comments.
fn block_or_line_doc(text: &str) -> Option<String> {
    let trimmed = text.trim_start();
    if let Some(after) = trimmed.strip_prefix("/*") {
        if let Some(end) = after.find("*/") {
            let body = &after[..end];
            let cleaned: Vec<String> = body
                .lines()
                .map(|l| l.trim_start().trim_start_matches('*').trim().to_string())
                .collect();
            return Some(cleaned.join("\n"));
        }
    }
    line_doc(text, &["//"])
}

/// A leading Python module docstring (`"""..."""` or `'''...'''`).
fn python_docstring(text: &str) -> Option<String> {
    // Skip leading blank lines, `#` comments, and `from __future__` lines.
    let mut rest = text;
    loop {
        let t = rest.trim_start();
        if t.starts_with('#') || t.starts_with("from __future__") {
            let nl = rest.find('\n')?;
            rest = &rest[nl + 1..];
            continue;
        }
        rest = t;
        break;
    }
    for quote in ["\"\"\"", "'''"] {
        if let Some(after) = rest.strip_prefix(quote) {
            if let Some(end) = after.find(quote) {
                return Some(after[..end].trim().to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_module_doc() {
        let text = "//! The auth module.\n//! Handles sessions.\nuse foo;\n";
        assert_eq!(
            leading_doc(FileLang::Rust, text).as_deref(),
            Some("The auth module.\nHandles sessions.")
        );
    }

    #[test]
    fn ts_jsdoc_block() {
        let text = "/**\n * Router for the API.\n */\nimport x from 'y';\n";
        assert_eq!(
            leading_doc(FileLang::TypeScript, text).as_deref(),
            Some("Router for the API.")
        );
    }

    #[test]
    fn python_docstring_extracted() {
        let text = "\"\"\"Database connection helpers.\"\"\"\nimport os\n";
        assert_eq!(
            leading_doc(FileLang::Python, text).as_deref(),
            Some("Database connection helpers.")
        );
    }

    #[test]
    fn no_doc_returns_none() {
        let text = "fn main() {}\n";
        assert_eq!(leading_doc(FileLang::Rust, text), None);
    }
}
