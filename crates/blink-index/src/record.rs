use std::fmt;
use std::path::Path;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::symbols::extract_symbols;

/// A source language label, derived purely from a file's extension. This is
/// deliberately broader than `blink_core::Language` (which describes a whole
/// *project's* primary language): the index labels every file individually and
/// covers a few languages the project detector doesn't, so `blink symbols`
/// works in a mixed-language repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileLang {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Go,
}

impl FileLang {
    /// Map a filename extension to a language, or `None` for files Blink does
    /// not extract symbols from (still indexed for size/change tracking).
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "rs" => Some(Self::Rust),
            "py" | "pyi" => Some(Self::Python),
            "ts" | "tsx" | "mts" | "cts" => Some(Self::TypeScript),
            "js" | "jsx" | "mjs" | "cjs" => Some(Self::JavaScript),
            "go" => Some(Self::Go),
            _ => None,
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Self::from_extension)
    }
}

impl fmt::Display for FileLang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            FileLang::Rust => "Rust",
            FileLang::Python => "Python",
            FileLang::TypeScript => "TypeScript",
            FileLang::JavaScript => "JavaScript",
            FileLang::Go => "Go",
        };
        write!(f, "{s}")
    }
}

/// The kind of a discovered top-level symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Class,
    Interface,
    TypeAlias,
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SymbolKind::Function => "fn",
            SymbolKind::Struct => "struct",
            SymbolKind::Enum => "enum",
            SymbolKind::Trait => "trait",
            SymbolKind::Class => "class",
            SymbolKind::Interface => "interface",
            SymbolKind::TypeAlias => "type",
        };
        write!(f, "{s}")
    }
}

/// A single top-level symbol declaration found in a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    /// 1-based line number of the declaration.
    pub line: usize,
}

/// One file's indexed metadata. `size` + `mtime` form the cheap change check:
/// if both match a prior record, the expensive `hash`/`symbols`/`lines` work is
/// skipped on the next update.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileRecord {
    /// Path relative to the index root, using `/` separators.
    pub path: String,
    pub size: u64,
    /// SHA-256 of the file contents, hex-encoded.
    pub hash: String,
    /// Modification time, whole seconds since the Unix epoch (0 if unavailable).
    pub mtime: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<FileLang>,
    /// Line count for text files with a known language, else 0.
    #[serde(default)]
    pub lines: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub symbols: Vec<Symbol>,
}

impl FileRecord {
    /// Fully (re)build a record for `root`/`rel`, reading and hashing the file.
    pub fn build(root: &Path, rel: &str) -> std::io::Result<Self> {
        let abs = root.join(rel);
        let meta = std::fs::metadata(&abs)?;
        let size = meta.len();
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let bytes = std::fs::read(&abs)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = hex::encode(hasher.finalize());

        let lang = FileLang::from_path(Path::new(rel));
        let (lines, symbols) = match (lang, std::str::from_utf8(&bytes)) {
            (Some(lang), Ok(text)) => (text.lines().count(), extract_symbols(lang, text)),
            _ => (0, Vec::new()),
        };

        Ok(Self {
            path: rel.to_string(),
            size,
            hash,
            mtime,
            lang,
            lines,
            symbols,
        })
    }
}

/// Normalize a path to use `/` separators so an index built on Windows and one
/// built on Unix compare and serialize identically.
pub(crate) fn normalize_rel(rel: &Path) -> String {
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
