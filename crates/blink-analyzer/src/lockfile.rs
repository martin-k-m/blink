use std::path::Path;

use blink_parser::LockedPackage;

/// A project's fully resolved dependency set, as read from whichever lockfile
/// the package manager wrote.
#[derive(Debug, Clone)]
pub struct ResolvedLockfile {
    /// The lockfile's filename, e.g. `"Cargo.lock"`. Reported verbatim so
    /// output always names the exact file the numbers came from.
    pub file: &'static str,
    /// Every package the lockfile resolved, at its exact version.
    pub packages: Vec<LockedPackage>,
    /// Names of the local project's *direct* dependencies. Empty when the
    /// format doesn't distinguish them.
    pub direct: Vec<String>,
    /// Whether Blink recovered package-to-package edges from this file.
    /// `false` means dependency paths can't be reconstructed and must not be
    /// shown.
    pub records_edges: bool,
}

/// Every lockfile format Blink can read, in the order they're looked for.
/// Cargo first because a Rust project may vendor a JS tool alongside it; npm's
/// own lock beats yarn's and pnpm's for the same reason.
pub const LOCKFILES: [&str; 4] = [
    "Cargo.lock",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
];

/// Load the resolved dependency set from whichever lockfile is present.
/// Returns `None` when none exists, since analysis that depends on *resolved*
/// versions can only run once a package manager has actually locked the
/// dependency tree.
pub fn load_lockfile(root: &Path) -> Option<ResolvedLockfile> {
    load_lockfile_from(root, &LOCKFILES)
}

/// Like [`load_lockfile`], but restricted to the given filenames. Used by the
/// security audit, which must read the lockfile belonging to the ecosystem
/// it's about to query — never, say, a `Cargo.lock` sitting next to the
/// `package.json` of a JS project.
pub fn load_lockfile_from(root: &Path, candidates: &[&'static str]) -> Option<ResolvedLockfile> {
    for file in candidates {
        let Ok(raw) = std::fs::read_to_string(root.join(file)) else {
            continue;
        };
        return Some(parse_lockfile(file, &raw));
    }
    None
}

/// Parse an already-read lockfile by name. Split out from [`load_lockfile`]
/// so the format handling can be tested without touching the filesystem.
pub fn parse_lockfile(file: &'static str, raw: &str) -> ResolvedLockfile {
    match file {
        "Cargo.lock" => {
            let packages = blink_parser::parse_cargo_lock(raw);
            // Cargo records workspace members as `[[package]]` entries with
            // no `source`; their dependency lists are the project's direct
            // dependencies, minus the workspace members themselves.
            let members: Vec<&str> = packages
                .iter()
                .filter(|p| p.local)
                .map(|p| p.name.as_str())
                .collect();
            let mut direct: Vec<String> = Vec::new();
            for member in packages.iter().filter(|p| p.local) {
                for dep in &member.dependencies {
                    if !members.contains(&dep.as_str()) && !direct.contains(dep) {
                        direct.push(dep.clone());
                    }
                }
            }
            ResolvedLockfile {
                file,
                packages,
                direct,
                records_edges: true,
            }
        }
        "package-lock.json" => ResolvedLockfile {
            file,
            packages: blink_parser::parse_npm_lock(raw),
            direct: blink_parser::parse_npm_lock_direct(raw),
            records_edges: true,
        },
        "yarn.lock" => ResolvedLockfile {
            file,
            packages: blink_parser::parse_yarn_lock(raw),
            direct: Vec::new(),
            records_edges: false,
        },
        "pnpm-lock.yaml" => ResolvedLockfile {
            file,
            packages: blink_parser::parse_pnpm_lock(raw),
            direct: Vec::new(),
            records_edges: false,
        },
        _ => ResolvedLockfile {
            file,
            packages: Vec::new(),
            direct: Vec::new(),
            records_edges: false,
        },
    }
}

/// Load just the resolved package list, for the counting/duplicate/size
/// analysis that doesn't care about edges.
pub fn load_locked_packages(root: &Path) -> Option<Vec<LockedPackage>> {
    load_lockfile(root).map(|lock| lock.packages)
}
