use blink_analyzer::{format_bytes, AnalysisReport};
use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, ContentArrangement, Table};

/// A compact stat table: direct/transitive/healthy dependency counts.
pub fn dependency_stats_table(report: &AnalysisReport) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Metric", "Count"]);

    table.add_row(vec![
        Cell::new("Direct"),
        Cell::new(report.dependency_counts.direct),
    ]);
    table.add_row(vec![
        Cell::new("Transitive"),
        Cell::new(
            report
                .dependency_counts
                .transitive
                .map(|n| n.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
        ),
    ]);
    table.add_row(vec![
        Cell::new("Healthy"),
        Cell::new(report.healthy_count()),
    ]);

    table.to_string()
}

/// A table of the largest direct dependencies by installed size. `None` if
/// nothing could be measured (dependencies not installed/built locally).
pub fn largest_packages_table(report: &AnalysisReport) -> Option<String> {
    if report.largest_dependencies.is_empty() {
        return None;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Package", "Size"]);

    for dep in &report.largest_dependencies {
        table.add_row(vec![
            Cell::new(&dep.name),
            Cell::new(format_bytes(dep.bytes)),
        ]);
    }

    Some(table.to_string())
}

#[cfg(test)]
mod tests {
    use blink_analyzer::{Analyzer, DependencyCounts};

    use super::*;

    #[test]
    fn dependency_stats_table_contains_counts() {
        let mut report = base_report();
        report.dependency_counts = DependencyCounts {
            direct: 12,
            transitive: Some(183),
        };

        let table = dependency_stats_table(&report);

        assert!(table.contains("Direct"));
        assert!(table.contains("12"));
        assert!(table.contains("183"));
    }

    #[test]
    fn largest_packages_table_is_none_when_empty() {
        let report = base_report();
        assert!(largest_packages_table(&report).is_none());
    }

    fn base_report() -> AnalysisReport {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"sample\"\n",
        )
        .unwrap();
        let project = blink_core::ProjectDetector::new()
            .detect(dir.path())
            .unwrap();
        Analyzer::new().analyze(&project, dir.path())
    }
}
