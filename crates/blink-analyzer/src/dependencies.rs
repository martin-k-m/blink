use blink_parser::LockedPackage;

/// Direct vs. transitive dependency counts for a project.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct DependencyCounts {
    pub direct: usize,
    /// `None` when no lockfile is present to resolve a transitive count
    /// from — there's nothing to distinguish "transitive" from "direct"
    /// without a package manager having actually locked the tree.
    pub transitive: Option<usize>,
}

/// Compute direct/transitive counts. `transitive` is derived as
/// `(total resolved packages) - direct`, which is an approximation: it
/// assumes every resolved package that isn't a direct dependency is
/// transitive, without walking the actual dependency edges. That's accurate
/// for the vast majority of projects and avoids reimplementing a package
/// manager's resolver just to count nodes.
pub fn count_dependencies(direct: usize, locked: Option<&[LockedPackage]>) -> DependencyCounts {
    let transitive = locked.map(|pkgs| pkgs.len().saturating_sub(direct));
    DependencyCounts { direct, transitive }
}
