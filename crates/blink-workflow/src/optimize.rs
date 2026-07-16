//! Rule-based project optimization report.
//!
//! Every check corresponds to a concrete, checkable condition — an actual
//! unused-dependency count, a real file-per-directory measurement, a file that
//! is or isn't on disk. Nothing here is a generic "you could go faster"
//! suggestion, and no build-speed improvement is ever claimed without
//! measurement (Blink measures none here, so it claims none).
//!
//! ## Score
//!
//! The score starts at 100 and deducts [`WARN_PENALTY`] points for each
//! category that raised a warning, clamped to `[0, 100]`. It is an
//! at-a-glance signal over the checks below, not a precise metric — the
//! per-category findings are the substance.

use std::collections::BTreeMap;
use std::path::Path;

use blink_analyzer::{format_bytes, AnalysisReport};
use blink_core::Project;
use blink_index::Index;

use crate::config_audit;
use crate::duplicates;

/// Points deducted from the score per warning category.
pub const WARN_PENALTY: u8 = 8;

/// A single source directory holding more than this many files is flagged as a
/// candidate for splitting. Heuristic, documented as such.
pub const LARGE_DIR_FILE_THRESHOLD: usize = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptStatus {
    Good,
    Warn,
}

/// One optimization category's result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptCheck {
    pub category: String,
    pub status: OptStatus,
    pub detail: String,
}

/// The full optimization report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptimizationReport {
    pub score: u8,
    pub checks: Vec<OptCheck>,
    /// Concrete next actions, each tied to a warning above.
    pub suggestions: Vec<String>,
}

impl OptimizationReport {
    pub fn warnings(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| c.status == OptStatus::Warn)
            .count()
    }
}

/// Produce an optimization report from already-computed analysis + index data.
pub fn optimize(
    project: &Project,
    report: &AnalysisReport,
    index: &Index,
    root: &Path,
) -> OptimizationReport {
    let mut checks = Vec::new();
    let mut suggestions = Vec::new();

    // --- Dependencies -------------------------------------------------------
    {
        let unused = report.unused.len();
        let dups = report.duplicates.len();
        let large = report.large_dependencies.len();
        if unused == 0 && dups == 0 && large == 0 {
            good(
                &mut checks,
                "Dependencies",
                "no unused, duplicate, or oversized packages",
            );
        } else {
            let mut parts = Vec::new();
            if unused > 0 {
                parts.push(format!("{unused} unused"));
            }
            if dups > 0 {
                parts.push(format!("{dups} duplicated"));
            }
            if large > 0 {
                parts.push(format!("{large} oversized"));
            }
            warn(&mut checks, "Dependencies", &parts.join(", "));
            if unused > 0 {
                suggestions.push(format!(
                    "Remove {unused} unused package{}: {}",
                    plural(unused),
                    report.unused.join(", ")
                ));
            }
            if dups > 0 {
                suggestions.push("Deduplicate packages resolved to multiple versions".to_string());
            }
        }
    }

    // --- Project Structure --------------------------------------------------
    {
        let counts = files_per_dir(index);
        let worst = counts
            .iter()
            .filter(|(dir, _)| !dir.is_empty())
            .max_by_key(|(_, n)| **n);
        match worst {
            Some((dir, n)) if *n > LARGE_DIR_FILE_THRESHOLD => {
                warn(
                    &mut checks,
                    "Project Structure",
                    &format!("{dir} contains {n} files"),
                );
                suggestions.push(format!("Split {dir} into smaller modules"));
            }
            _ => good(&mut checks, "Project Structure", "no oversized directories"),
        }
    }

    // --- Duplicate Files ----------------------------------------------------
    {
        let groups = duplicates::find(index);
        if groups.is_empty() {
            good(&mut checks, "Duplicate Files", "no identical files found");
        } else {
            let wasted = duplicates::total_wasted(&groups);
            warn(
                &mut checks,
                "Duplicate Files",
                &format!(
                    "{} group{} of identical files ({} reclaimable)",
                    groups.len(),
                    plural(groups.len()),
                    format_bytes(wasted)
                ),
            );
            suggestions
                .push("Consolidate duplicate files reported by `blink duplicates`".to_string());
        }
    }

    // --- Tests --------------------------------------------------------------
    {
        if has_tests(index) {
            good(&mut checks, "Tests", "test files detected");
        } else {
            warn(&mut checks, "Tests", "no test files detected");
            suggestions.push("Add tests to cover the project's behavior".to_string());
        }
    }

    // --- Documentation ------------------------------------------------------
    // --- Configuration ------------------------------------------------------
    {
        let audit = config_audit::audit(root);
        let missing_docs: Vec<&str> = audit
            .iter()
            .filter(|i| !i.present && matches!(i.name.as_str(), "README" | "CONTRIBUTING"))
            .map(|i| i.name.as_str())
            .collect();
        if missing_docs.is_empty() {
            good(&mut checks, "Documentation", "README present");
        } else {
            warn(
                &mut checks,
                "Documentation",
                &format!("missing {}", missing_docs.join(", ")),
            );
            for name in missing_docs {
                suggestions.push(format!("Add a {name} file"));
            }
        }

        let missing_config: Vec<&str> = audit
            .iter()
            .filter(|i| !i.present && matches!(i.name.as_str(), ".gitignore" | "CI configuration"))
            .map(|i| i.name.as_str())
            .collect();
        if missing_config.is_empty() {
            good(&mut checks, "Configuration", ".gitignore and CI present");
        } else {
            warn(
                &mut checks,
                "Configuration",
                &format!("missing {}", missing_config.join(", ")),
            );
            for name in missing_config {
                suggestions.push(format!("Add {name}"));
            }
        }
    }

    let warnings = checks
        .iter()
        .filter(|c| c.status == OptStatus::Warn)
        .count();
    let score =
        100u8.saturating_sub(WARN_PENALTY.saturating_mul(warnings.min(u8::MAX as usize) as u8));

    // Mention the project so an all-good report still names what it looked at.
    let _ = project;

    OptimizationReport {
        score,
        checks,
        suggestions,
    }
}

fn good(checks: &mut Vec<OptCheck>, category: &str, detail: &str) {
    checks.push(OptCheck {
        category: category.to_string(),
        status: OptStatus::Good,
        detail: detail.to_string(),
    });
}

fn warn(checks: &mut Vec<OptCheck>, category: &str, detail: &str) {
    checks.push(OptCheck {
        category: category.to_string(),
        status: OptStatus::Warn,
        detail: detail.to_string(),
    });
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

/// Count files whose immediate parent directory is each path prefix. The key is
/// the `/`-joined parent (empty string for root-level files).
fn files_per_dir(index: &Index) -> BTreeMap<String, usize> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for path in index.files.keys() {
        let parent = match path.rsplit_once('/') {
            Some((dir, _)) => dir.to_string(),
            None => String::new(),
        };
        *counts.entry(parent).or_insert(0) += 1;
    }
    counts
}

/// Heuristic test detection: a path that lives in a `tests`/`__tests__`
/// directory, or a filename following a common test-file convention.
fn has_tests(index: &Index) -> bool {
    index.files.keys().any(|path| {
        let lower = path.to_ascii_lowercase();
        let file = lower.rsplit('/').next().unwrap_or(&lower);
        lower
            .split('/')
            .any(|c| c == "tests" || c == "__tests__" || c == "test")
            || file.contains(".test.")
            || file.contains(".spec.")
            || file.contains("_test.")
            || file.starts_with("test_")
    })
}
