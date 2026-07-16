use blink_analyzer::AnalysisReport;
use blink_core::Project;
use serde::Serialize;

use crate::label::project_type_label;

/// A JSON-serializable summary of a project scan + analysis, for
/// `blink analyze --json`. Every field here is one already shown in the
/// terminal report — this isn't a separate, richer data source.
#[derive(Debug, Clone, Serialize)]
pub struct JsonReport {
    pub project: String,
    #[serde(rename = "type")]
    pub project_type: String,
    pub files: usize,
    pub dependencies: JsonDependencies,
    pub health: JsonHealth,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
    pub analysis_time_ms: u128,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonDependencies {
    pub direct: usize,
    pub transitive: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonHealth {
    pub score: u8,
    pub healthy_packages: usize,
}

/// Build a [`JsonReport`] from a detected project and its analysis.
pub fn to_json_report(project: &Project, report: &AnalysisReport) -> JsonReport {
    JsonReport {
        project: project.name.clone(),
        project_type: project_type_label(project),
        files: project.file_count,
        dependencies: JsonDependencies {
            direct: report.dependency_counts.direct,
            transitive: report.dependency_counts.transitive,
        },
        health: JsonHealth {
            score: report.health_score(),
            healthy_packages: report.healthy_count(),
        },
        issues: crate::issues::issues(report)
            .into_iter()
            .map(|issue| issue.summary)
            .collect(),
        suggestions: report.recommendations(),
        analysis_time_ms: report.elapsed_ms,
    }
}

#[cfg(test)]
mod tests {
    use blink_core::ProjectDetector;
    use tempfile::TempDir;

    use super::to_json_report;
    use blink_analyzer::Analyzer;

    #[test]
    fn serializes_expected_fields() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"name": "my-app", "dependencies": {"react": "^18.0.0"}}"#,
        )
        .unwrap();

        let project = ProjectDetector::new().detect(dir.path()).unwrap();
        let report = Analyzer::new().analyze(&project, dir.path());
        let json = to_json_report(&project, &report);

        let serialized = serde_json::to_string(&json).unwrap();
        assert!(serialized.contains("\"project\":\"my-app\""));
        assert!(serialized.contains("\"type\":\"React + JavaScript\""));
        assert!(serialized.contains("\"direct\":1"));
    }
}
