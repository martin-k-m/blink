use blink_analyzer::{format_bytes, AnalysisReport, HealthReport};
use blink_core::Project;

use crate::label::project_type_label;

/// Render a full, self-contained HTML report (inline CSS, no external
/// assets) — suitable for archiving, emailing, or publishing as a static
/// page.
pub fn render_html(project: &Project, report: &AnalysisReport, health: &HealthReport) -> String {
    let issues = crate::issues::issues(report);
    let recommendations = report.recommendations();

    let largest_rows: String = report
        .largest_dependencies
        .iter()
        .map(|dep| {
            format!(
                "<tr><td>{}</td><td>{}</td></tr>",
                escape(&dep.name),
                format_bytes(dep.bytes)
            )
        })
        .collect();

    let issue_items: String = if issues.is_empty() {
        "<li>No issues detected.</li>".to_string()
    } else {
        issues
            .iter()
            .map(|i| format!("<li>{}</li>", escape(&i.summary)))
            .collect()
    };

    let recommendation_items: String = if recommendations.is_empty() {
        "<li>No recommendations.</li>".to_string()
    } else {
        recommendations
            .iter()
            .map(|r| format!("<li>{}</li>", escape(r)))
            .collect()
    };

    let transitive = report
        .dependency_counts
        .transitive
        .map(|n| n.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Blink Project Report: {name}</title>
<style>
  body {{ background: #0b0b0c; color: #f5f5f5; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; max-width: 720px; margin: 2rem auto; padding: 0 1rem; }}
  h1 {{ color: #ff2d8d; }}
  h2 {{ border-bottom: 1px solid #333; padding-bottom: 0.25rem; }}
  table {{ border-collapse: collapse; width: 100%; margin: 1rem 0; }}
  td, th {{ border: 1px solid #333; padding: 0.4rem 0.6rem; text-align: left; }}
  .bar {{ background: #222; border-radius: 4px; overflow: hidden; height: 1.25rem; margin: 0.5rem 0; }}
  .bar-fill {{ background: #ff2d8d; height: 100%; }}
  .meta {{ color: #9a9a9a; }}
</style>
</head>
<body>
  <h1>&#9889; Blink Project Report</h1>
  <p class="meta">{project_type}</p>

  <h2>Health: {health}%</h2>
  <div class="bar"><div class="bar-fill" style="width: {health}%"></div></div>

  <h2>Dependencies</h2>
  <table>
    <tr><th>Direct</th><td>{direct}</td></tr>
    <tr><th>Transitive</th><td>{transitive}</td></tr>
    <tr><th>Healthy</th><td>{healthy}</td></tr>
  </table>

  {largest_section}

  <h2>Issues ({issue_count})</h2>
  <ul>{issue_items}</ul>

  <h2>Recommendations ({rec_count})</h2>
  <ul>{recommendation_items}</ul>
</body>
</html>
"#,
        name = escape(&project.name),
        project_type = escape(&project_type_label(project)),
        health = health.overall,
        direct = report.dependency_counts.direct,
        healthy = report.healthy_count(),
        largest_section = if largest_rows.is_empty() {
            String::new()
        } else {
            format!(
                "<h2>Largest Packages</h2>\n  <table><tr><th>Package</th><th>Size</th></tr>{largest_rows}</table>"
            )
        },
        issue_count = issues.len(),
        rec_count = recommendations.len(),
    )
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::render_html;
    use blink_analyzer::{compute_health, Analyzer};
    use blink_core::ProjectDetector;

    #[test]
    fn html_report_is_well_formed_and_includes_key_sections() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"sample\"\n",
        )
        .unwrap();

        let project = ProjectDetector::new().detect(dir.path()).unwrap();
        let report = Analyzer::new().analyze(&project, dir.path());
        let health = compute_health(&report, dir.path());

        let html = render_html(&project, &report, &health);

        assert!(html.starts_with("<!doctype html>"));
        assert!(html.contains("<title>Blink Project Report: sample</title>"));
        assert!(html.contains(&format!("Health: {}%", health.overall)));
        assert!(html.ends_with("</html>\n"));
    }

    #[test]
    fn html_escapes_project_name() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"name": "<script>alert(1)</script>"}"#,
        )
        .unwrap();

        let project = ProjectDetector::new().detect(dir.path()).unwrap();
        let report = Analyzer::new().analyze(&project, dir.path());
        let health = compute_health(&report, dir.path());

        let html = render_html(&project, &report, &health);

        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
