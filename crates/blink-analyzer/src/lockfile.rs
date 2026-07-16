use std::path::Path;

use blink_parser::LockedPackage;

/// Load the resolved package list from whichever lockfile is present
/// (`Cargo.lock` or `package-lock.json`). Returns `None` when neither
/// exists, since duplicate/size/count analysis that depends on *resolved*
/// versions can only run once a package manager has actually locked the
/// dependency tree.
pub fn load_locked_packages(root: &Path) -> Option<Vec<LockedPackage>> {
    if let Ok(raw) = std::fs::read_to_string(root.join("Cargo.lock")) {
        return Some(blink_parser::parse_cargo_lock(&raw));
    }
    if let Ok(raw) = std::fs::read_to_string(root.join("package-lock.json")) {
        return Some(blink_parser::parse_npm_lock(&raw));
    }
    None
}
