use blink_analyzer::{format_bytes, AnalysisReport, HealthReport};
use blink_core::Project;

use crate::label::project_type_label;

/// Render a full Markdown report: suitable for pasting into a PR
/// description, a wiki page, or a CI job summary.
pub fn render_markdown(
    project: &Project,
    report: &AnalysisReport,
    health: &HealthReport,
) -> String {
    let mut out = String::new();

    out.push_str(&format!("# Blink Project Report: {}\n\n", project.name));
    out.push_str(&format!("**Type:** {}\n\n", project_type_label(project)));
    out.push_str(&format!("**Health:** {}%\n\n", health.overall));

    out.push_str("## Dependencies\n\n");
    out.push_str("| Metric | Count |\n");
    out.push_str("| --- | --- |\n");
    out.push_str(&format!(
        "| Direct | {} |\n",
        report.dependency_counts.direct
    ));
    out.push_str(&format!(
        "| Transitive | {} |\n",
        report
            .dependency_counts
            .transitive
            .map(|n| n.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    ));
    out.push_str(&format!("| Healthy | {} |\n\n", report.healthy_count()));

    if !report.largest_dependencies.is_empty() {
        out.push_str("## Largest Packages\n\n");
        out.push_str("| Package | Size |\n");
        out.push_str("| --- | --- |\n");
        for dep in &report.largest_dependencies {
            out.push_str(&format!("| {} | {} |\n", dep.name, format_bytes(dep.bytes)));
        }
        out.push('\n');
    }

    let issues = crate::issues::issues(report);
    out.push_str(&format!("## Issues ({})\n\n", issues.len()));
    if issues.is_empty() {
        out.push_str("No issues detected.\n\n");
    } else {
        for issue in &issues {
            out.push_str(&format!("- {}\n", issue.summary));
        }
        out.push('\n');
    }

    let recommendations = report.recommendations();
    out.push_str(&format!(
        "## Recommendations ({})\n\n",
        recommendations.len()
    ));
    if recommendations.is_empty() {
        out.push_str("No recommendations.\n\n");
    } else {
        for rec in &recommendations {
            out.push_str(&format!("- {rec}\n"));
        }
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::render_markdown;
    use blink_analyzer::{compute_health, Analyzer};
    use blink_core::ProjectDetector;

    #[test]
    fn markdown_report_includes_key_sections() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"sample\"\n",
        )
        .unwrap();

        let project = ProjectDetector::new().detect(dir.path()).unwrap();
        let report = Analyzer::new().analyze(&project, dir.path());
        let health = compute_health(&report, dir.path());

        let markdown = render_markdown(&project, &report, &health);

        assert!(markdown.starts_with("# Blink Project Report: sample"));
        assert!(markdown.contains("## Dependencies"));
        assert!(markdown.contains("## Issues"));
        assert!(markdown.contains("## Recommendations"));
    }
}
