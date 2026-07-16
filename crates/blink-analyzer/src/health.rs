use std::path::Path;

use crate::report::AnalysisReport;

/// A breakdown of project health into three independently measurable
/// sub-scores, each 0-100, plus an overall average. None of these are
/// invented: every point comes from a concrete, checkable fact about the
/// project (a file exists, a dependency is flagged, ...). See
/// `docs/analysis.md` for the exact rubric.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthReport {
    pub overall: u8,
    pub dependencies: u8,
    pub configuration: u8,
    pub code_organization: u8,
    pub suggestions: Vec<String>,
}

/// Compute a [`HealthReport`] for the project rooted at `root`, whose
/// dependency findings are already captured in `analysis`.
pub fn compute_health(analysis: &AnalysisReport, root: &Path) -> HealthReport {
    let dependencies = analysis.health_score();
    let (configuration, mut config_suggestions) = score_configuration(root);
    let (code_organization, mut organization_suggestions) = score_code_organization(root);

    let overall = ((dependencies as u32 + configuration as u32 + code_organization as u32) / 3)
        .min(100) as u8;

    let mut suggestions = analysis.recommendations();
    suggestions.append(&mut config_suggestions);
    suggestions.append(&mut organization_suggestions);

    HealthReport {
        overall,
        dependencies,
        configuration,
        code_organization,
        suggestions,
    }
}

/// Configuration hygiene: 25 points each for a `blink.toml`, a lockfile,
/// a `.gitignore`, and a README.
fn score_configuration(root: &Path) -> (u8, Vec<String>) {
    let mut score: u32 = 0;
    let mut suggestions = Vec::new();

    if blink_core::BlinkConfig::exists(root) {
        score += 25;
    } else {
        suggestions.push("Run `blink init` to add a blink.toml".to_string());
    }

    let has_lockfile = [
        "Cargo.lock",
        "package-lock.json",
        "pnpm-lock.yaml",
        "yarn.lock",
    ]
    .iter()
    .any(|f| root.join(f).is_file());
    if has_lockfile {
        score += 25;
    } else {
        suggestions.push("Add a lockfile for reproducible installs".to_string());
    }

    let has_gitignore = root.join(".gitignore").is_file();
    if has_gitignore {
        score += 25;
    } else {
        suggestions.push("Add a .gitignore".to_string());
    }

    let has_readme = ["README.md", "README", "readme.md"]
        .iter()
        .any(|f| root.join(f).is_file());
    if has_readme {
        score += 25;
    } else {
        suggestions.push("Add a README".to_string());
    }

    (score as u8, suggestions)
}

/// Project structure: 40 points for a dedicated source directory, 30 for a
/// tests directory, 30 for a docs directory.
fn score_code_organization(root: &Path) -> (u8, Vec<String>) {
    let mut score: u32 = 0;
    let mut suggestions = Vec::new();

    let has_src = root.join("src").is_dir() || root.join("crates").is_dir();
    if has_src {
        score += 40;
    } else {
        suggestions.push("Organize source files under a src/ directory".to_string());
    }

    let has_tests = ["tests", "test", "__tests__"]
        .iter()
        .any(|d| root.join(d).is_dir());
    if has_tests {
        score += 30;
    } else {
        suggestions.push("Add a tests/ directory".to_string());
    }

    let has_docs = root.join("docs").is_dir();
    if has_docs {
        score += 30;
    } else {
        suggestions.push("Add a docs/ directory".to_string());
    }

    (score as u8, suggestions)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::compute_health;
    use crate::Analyzer;

    #[test]
    fn well_organized_project_scores_highly_on_configuration_and_structure() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"sample\"\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("Cargo.lock"), "").unwrap();
        std::fs::write(dir.path().join(".gitignore"), "target\n").unwrap();
        std::fs::write(dir.path().join("README.md"), "# sample\n").unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::create_dir_all(dir.path().join("tests")).unwrap();
        std::fs::create_dir_all(dir.path().join("docs")).unwrap();
        blink_core::BlinkConfig::new("sample")
            .write(dir.path())
            .unwrap();

        let project = blink_core::ProjectDetector::new()
            .detect(dir.path())
            .unwrap();
        let analysis = Analyzer::new().analyze(&project, dir.path());
        let health = compute_health(&analysis, dir.path());

        assert_eq!(health.configuration, 100);
        assert_eq!(health.code_organization, 100);
    }

    #[test]
    fn bare_project_scores_zero_on_configuration_and_structure() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"sample\"\n",
        )
        .unwrap();

        let project = blink_core::ProjectDetector::new()
            .detect(dir.path())
            .unwrap();
        let analysis = Analyzer::new().analyze(&project, dir.path());
        let health = compute_health(&analysis, dir.path());

        assert_eq!(health.configuration, 0);
        assert_eq!(health.code_organization, 0);
        assert!(!health.suggestions.is_empty());
    }
}
