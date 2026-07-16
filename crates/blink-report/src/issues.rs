use blink_analyzer::AnalysisReport;

/// A single detected issue, as plain text with no icon or color applied —
/// callers decide how to present it (an icon, a color, a table row, ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub summary: String,
}

/// Summarize every issue an [`AnalysisReport`] found, one line each. Empty
/// when the project is clean.
pub fn issues(report: &AnalysisReport) -> Vec<Issue> {
    let mut issues = Vec::new();

    if !report.unused.is_empty() {
        issues.push(Issue {
            summary: format!("Unused dependency detected ({})", report.unused.len()),
        });
    }
    if !report.duplicates.is_empty() {
        issues.push(Issue {
            summary: format!(
                "Duplicate package versions detected ({})",
                report.duplicates.len()
            ),
        });
    }
    if !report.large_dependencies.is_empty() {
        issues.push(Issue {
            summary: format!(
                "Large dependency detected ({})",
                report.large_dependencies.len()
            ),
        });
    }
    if report.outdated_checked && !report.outdated.is_empty() {
        issues.push(Issue {
            summary: format!("Outdated packages detected ({})", report.outdated.len()),
        });
    }

    issues
}

#[cfg(test)]
mod tests {
    use blink_core::ProjectDetector;
    use tempfile::TempDir;

    use super::issues;
    use blink_analyzer::Analyzer;

    #[test]
    fn no_issues_for_a_clean_project() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"sample\"\n",
        )
        .unwrap();

        let project = ProjectDetector::new().detect(dir.path()).unwrap();
        let report = Analyzer::new().analyze(&project, dir.path());

        assert!(issues(&report).is_empty());
    }

    #[test]
    fn reports_duplicate_versions_as_an_issue() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"sample\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("Cargo.lock"),
            r#"
                [[package]]
                name = "syn"
                version = "1.0.0"

                [[package]]
                name = "syn"
                version = "2.0.0"
            "#,
        )
        .unwrap();

        let project = ProjectDetector::new().detect(dir.path()).unwrap();
        let report = Analyzer::new().analyze(&project, dir.path());

        let found = issues(&report);
        assert_eq!(found.len(), 1);
        assert!(found[0].summary.contains("Duplicate"));
    }
}
