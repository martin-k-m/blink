//! Blink's incremental project index.
//!
//! The index is a per-project, on-disk record of every file's size, hash,
//! modification time, language, line count, and top-level symbols. It exists so
//! Blink stops rescanning and re-hashing an entire repository on every run:
//! [`Index::refresh`] re-processes only the files whose size or mtime changed
//! and reuses stored records for the rest, then answers `search`, `symbols`,
//! `hotspots`, and `status` queries from memory.
//!
//! Nothing here is estimated. Sizes and line counts are read from the files;
//! [`RefreshStats`] counts the files actually walked; symbols come from a
//! conservative line scanner (see [`symbols`]) that prefers to miss an oddly
//! formatted declaration over inventing one.

mod error;
mod index;
mod record;
mod symbols;

#[cfg(test)]
mod tests;

pub use error::{IndexError, Result};
pub use index::{Index, RefreshStats, INDEX_DIR, INDEX_FILE, INDEX_VERSION};
pub use record::{FileLang, FileRecord, Symbol, SymbolKind};
pub use symbols::extract_symbols;
