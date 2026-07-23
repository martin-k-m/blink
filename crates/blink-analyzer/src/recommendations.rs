use std::path::Path;

use blink_core::BlinkConfig;

use crate::report::AnalysisReport;
use crate::security::VulnerablePackage;

/// Whether a [`Recommendation`] reflects something healthy, something that
/// needs attention, or something Blink didn't check (because the relevant
/// flag wasn't passed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Status {
    Ok,
    Warning,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum RecommendationCategory {
    Performance,
    Maintenance,
    Security,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Recommendation {
    pub category: RecommendationCategory,
    pub status: Status,
    pub message: String,
}

/// Groups dependency/configuration/security findings into deterministic,
/// rule-based recommendations. Every rule here checks a concrete fact
/// (a config value, a finding already in an [`AnalysisReport`], a
/// vulnerability lookup result) — there's no scoring or fuzziness beyond
/// what [`AnalysisReport::health_score`] already documents.
pub struct RecommendationEngine;

impl RecommendationEngine {
    /// Evaluate all rules. `vulnerabilities` is `None` when the security
    /// check wasn't run (it requires `--online`, like the outdated-package
    /// check); pass `Some(&[])` for "checked, found nothing."
    pub fn evaluate(
        analysis: &AnalysisReport,
        root: &Path,
        vulnerabilities: Option<&[VulnerablePackage]>,
    ) -> Vec<Recommendation> {
        let mut recs = Vec::new();
        recs.extend(performance_rules(analysis, root));
        recs.extend(maintenance_rules(analysis));
        recs.push(security_rule(vulnerabilities));
        recs
    }
}

fn performance_rules(analysis: &AnalysisReport, root: &Path) -> Vec<Recommendation> {
    let cache_enabled = BlinkConfig::load(root)
        .map(|c| c.optimization.cache)
        .unwrap_or(true);

    let mut recs = vec![Recommendation {
        category: RecommendationCategory::Performance,
        status: if cache_enabled {
            Status::Ok
        } else {
            Status::Warning
        },
        message: if cache_enabled {
            "Dependency caching enabled".to_string()
        } else {
            "Dependency caching disabled in blink.toml".to_string()
        },
    }];

    if !analysis.large_dependencies.is_empty() {
        recs.push(Recommendation {
            category: RecommendationCategory::Performance,
            status: Status::Warning,
            message: format!(
                "{} large {} found",
                analysis.large_dependencies.len(),
                if analysis.large_dependencies.len() == 1 {
                    "dependency"
                } else {
                    "dependencies"
                }
            ),
        });
    }

    recs
}

fn maintenance_rules(analysis: &AnalysisReport) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    recs.push(if analysis.unused.is_empty() {
        Recommendation {
            category: RecommendationCategory::Maintenance,
            status: Status::Ok,
            message: "No unused dependencies detected".to_string(),
        }
    } else {
        Recommendation {
            category: RecommendationCategory::Maintenance,
            status: Status::Warning,
            message: format!("{} unused dependencies found", analysis.unused.len()),
        }
    });

    if !analysis.duplicates.is_empty() {
        recs.push(Recommendation {
            category: RecommendationCategory::Maintenance,
            status: Status::Warning,
            message: format!(
                "{} packages with duplicate versions found",
                analysis.duplicates.len()
            ),
        });
    }

    recs.push(if !analysis.outdated_checked {
        Recommendation {
            category: RecommendationCategory::Maintenance,
            status: Status::Unknown,
            message: "Outdated packages: unknown (run with --online to check)".to_string(),
        }
    } else if analysis.outdated.is_empty() {
        Recommendation {
            category: RecommendationCategory::Maintenance,
            status: Status::Ok,
            message: "All dependencies up to date".to_string(),
        }
    } else {
        Recommendation {
            category: RecommendationCategory::Maintenance,
            status: Status::Warning,
            message: format!("{} outdated packages found", analysis.outdated.len()),
        }
    });

    recs
}

fn security_rule(vulnerabilities: Option<&[VulnerablePackage]>) -> Recommendation {
    match vulnerabilities {
        None => Recommendation {
            category: RecommendationCategory::Security,
            status: Status::Unknown,
            message: "Vulnerabilities: unknown (needs --online and a reachable osv.dev)"
                .to_string(),
        },
        Some([]) => Recommendation {
            category: RecommendationCategory::Security,
            status: Status::Ok,
            message: "No known dependency vulnerabilities detected".to_string(),
        },
        Some(vulns) => Recommendation {
            category: RecommendationCategory::Security,
            status: Status::Warning,
            message: format!(
                "{} dependencies with known vulnerabilities found",
                vulns.len()
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::Analyzer;

    fn analysis_for(dir: &TempDir) -> AnalysisReport {
        let project = blink_core::ProjectDetector::new()
            .detect(dir.path())
            .unwrap();
        Analyzer::new().analyze(&project, dir.path())
    }

    #[test]
    fn security_unknown_when_not_checked() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"sample\"\n",
        )
        .unwrap();
        let analysis = analysis_for(&dir);

        let recs = RecommendationEngine::evaluate(&analysis, dir.path(), None);
        let security = recs
            .iter()
            .find(|r| r.category == RecommendationCategory::Security)
            .unwrap();

        assert_eq!(security.status, Status::Unknown);
    }

    #[test]
    fn security_ok_when_checked_and_clean() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"sample\"\n",
        )
        .unwrap();
        let analysis = analysis_for(&dir);

        let recs = RecommendationEngine::evaluate(&analysis, dir.path(), Some(&[]));
        let security = recs
            .iter()
            .find(|r| r.category == RecommendationCategory::Security)
            .unwrap();

        assert_eq!(security.status, Status::Ok);
    }

    #[test]
    fn performance_warns_when_cache_disabled() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"sample\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("blink.toml"),
            "[project]\nname = \"sample\"\n\n[optimization]\ncache = false\n",
        )
        .unwrap();
        let analysis = analysis_for(&dir);

        let recs = RecommendationEngine::evaluate(&analysis, dir.path(), None);
        let perf = recs
            .iter()
            .find(|r| r.category == RecommendationCategory::Performance)
            .unwrap();

        assert_eq!(perf.status, Status::Warning);
    }
}
