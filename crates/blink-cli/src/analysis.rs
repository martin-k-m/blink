use std::path::Path;
use std::time::Instant;

use blink_analyzer::{AnalysisReport, Analyzer};
use blink_cache::{AnalysisCache, Cache};
use blink_core::Project;

/// The result of [`analyze_cached`]: the report, whether it came from
/// Blink's global analysis cache, and how long *this* call took (which is
/// not the same as `report.elapsed_ms` on a cache hit — that field is
/// whatever the original, uncached run measured).
pub struct AnalyzeOutcome {
    pub report: AnalysisReport,
    pub from_cache: bool,
    pub elapsed_ms: u128,
}

/// Run the analyzer, transparently reusing Blink's global per-user
/// analysis cache when the project's files haven't changed since the last
/// run in `online: false` mode.
///
/// `online: true` always recomputes: registry/vulnerability lookups can go
/// stale independent of whether the project's own files changed, so
/// caching that result risks reporting an outdated "up to date" or "no
/// vulnerabilities found" verdict.
pub fn analyze_cached(project: &Project, root: &Path, online: bool) -> AnalyzeOutcome {
    let start = Instant::now();

    if online {
        return AnalyzeOutcome {
            report: Analyzer::new().online(true).analyze(project, root),
            from_cache: false,
            elapsed_ms: start.elapsed().as_millis(),
        };
    }

    let cache = AnalysisCache::open().ok();
    let snapshot = Cache::scan(root);

    if let Some(cache) = &cache {
        if let Some(report) = cache.get::<AnalysisReport>(root, &snapshot) {
            return AnalyzeOutcome {
                report,
                from_cache: true,
                elapsed_ms: start.elapsed().as_millis(),
            };
        }
    }

    let report = Analyzer::new().analyze(project, root);
    if let Some(cache) = &cache {
        let _ = cache.set(root, snapshot, &report);
    }
    AnalyzeOutcome {
        report,
        from_cache: false,
        elapsed_ms: start.elapsed().as_millis(),
    }
}
