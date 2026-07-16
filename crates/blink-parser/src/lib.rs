//! Reads and parses project manifest and lockfile formats into plain
//! structured data. This crate knows nothing about Blink's domain concepts
//! (projects, frameworks, dependency health) — it only knows file formats.

mod dependency;
mod error;
mod lockfile;
mod manifest;

#[cfg(test)]
mod tests;

pub use dependency::RawDependency;
pub use error::{ParserError, Result};
pub use lockfile::{parse_cargo_lock, parse_npm_lock, LockedPackage};
pub use manifest::{
    parse_cargo_manifest, parse_package_json, parse_requirements_txt, CargoManifest,
    PackageJsonManifest,
};
