//! Blink's project **context graph**.
//!
//! This crate is the heart of Blink's context engine: it unifies the three
//! things Blink already knows about a project — its detected identity
//! (`blink-core`), its indexed files and symbols (`blink-index`), and its
//! declared dependencies — into one structured, serializable
//! [`ContextGraph`]. On top of that it resolves the *references* between files
//! (who imports whom) and groups files into logical [`Area`]s.
//!
//! Nothing here is fabricated. Every count, size, symbol, and reference is
//! measured or resolved from real files; the only interpretive step is how
//! files are grouped into areas (see [`build::area_of`]). Import resolution is
//! deliberately conservative — an import that can't be resolved to an actual
//! project file is never turned into an invented edge (see [`imports`]).
//!
//! The graph is consumed by `blink context`/`map`/`explain`, by `blink-query`
//! for structured search, and by `blink-export` for serialization.

mod build;
mod explain;
pub mod imports;
mod model;

#[cfg(test)]
mod tests;

pub use build::area_of;
pub use explain::FileExplanation;
pub use imports::ScanResult;
pub use model::{
    Area, AreaEdge, CommandNode, ConfigInfo, ContextGraph, DependencyNode, FileNode, ProjectInfo,
    Reference, Stats, SymbolRef,
};
