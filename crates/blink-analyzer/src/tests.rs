use std::fs;

use blink_core::ProjectDetector;
use tempfile::TempDir;

use crate::Analyzer;

fn write(dir: &TempDir, name: &str, contents: &str) {
    let path = dir.path().join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[test]
fn detects_unused_runtime_dependency() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "package.json",
        r#"{
            "name": "sample",
            "dependencies": { "react": "^18.0.0", "lodash": "^4.17.21" }
        }"#,
    );
    write(
        &dir,
        "src/App.jsx",
        "import React from 'react';\nexport default function App() { return null; }",
    );

    let project = ProjectDetector::new().detect(dir.path()).unwrap();
    let report = Analyzer::new().analyze(&project, dir.path());

    assert_eq!(report.unused, vec!["lodash".to_string()]);
    assert_eq!(report.dependency_graph.direct_count(), 2);
}

#[test]
fn detects_duplicate_versions_from_cargo_lock() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "Cargo.toml",
        r#"
            [package]
            name = "sample"
            version = "0.1.0"

            [dependencies]
            serde = "1"
        "#,
    );
    write(
        &dir,
        "Cargo.lock",
        r#"
            [[package]]
            name = "serde"
            version = "1.0.190"

            [[package]]
            name = "syn"
            version = "1.0.0"

            [[package]]
            name = "syn"
            version = "2.0.0"
        "#,
    );

    let project = ProjectDetector::new().detect(dir.path()).unwrap();
    let report = Analyzer::new().analyze(&project, dir.path());

    assert_eq!(report.duplicates.len(), 1);
    assert_eq!(report.duplicates[0].name, "syn");
    assert_eq!(report.duplicates[0].versions.len(), 2);
}

#[test]
fn recommendations_are_derived_from_findings() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "package.json",
        r#"{"name": "sample", "dependencies": {"left-pad": "^1.0.0"}}"#,
    );

    let project = ProjectDetector::new().detect(dir.path()).unwrap();
    let report = Analyzer::new().analyze(&project, dir.path());

    assert!(report
        .recommendations()
        .iter()
        .any(|r| r.contains("left-pad")));
}

#[test]
fn offline_analysis_never_reports_outdated_packages() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "package.json",
        r#"{"name": "sample", "dependencies": {"react": "0.0.1"}}"#,
    );

    let project = ProjectDetector::new().detect(dir.path()).unwrap();
    let report = Analyzer::new().analyze(&project, dir.path());

    assert!(!report.outdated_checked);
    assert!(report.outdated.is_empty());
}

#[test]
fn dependency_counts_without_lockfile_have_no_transitive_count() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "package.json",
        r#"{"name": "sample", "dependencies": {"react": "^18.0.0"}}"#,
    );

    let project = ProjectDetector::new().detect(dir.path()).unwrap();
    let report = Analyzer::new().analyze(&project, dir.path());

    assert_eq!(report.dependency_counts.direct, 1);
    assert_eq!(report.dependency_counts.transitive, None);
}

#[test]
fn dependency_counts_derive_transitive_from_lockfile() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "Cargo.toml",
        r#"
            [package]
            name = "sample"
            version = "0.1.0"

            [dependencies]
            serde = "1"
        "#,
    );
    write(
        &dir,
        "Cargo.lock",
        r#"
            [[package]]
            name = "serde"
            version = "1.0.190"

            [[package]]
            name = "serde_derive"
            version = "1.0.190"

            [[package]]
            name = "syn"
            version = "2.0.0"
        "#,
    );

    let project = ProjectDetector::new().detect(dir.path()).unwrap();
    let report = Analyzer::new().analyze(&project, dir.path());

    assert_eq!(report.dependency_counts.direct, 1);
    assert_eq!(report.dependency_counts.transitive, Some(2));
}

#[test]
fn health_score_starts_at_100_with_no_issues() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "package.json",
        r#"{"name": "sample", "dependencies": {"react": "^18.0.0"}}"#,
    );
    write(
        &dir,
        "src/App.jsx",
        "import React from 'react';\nexport default function App() { return null; }",
    );

    let project = ProjectDetector::new().detect(dir.path()).unwrap();
    let report = Analyzer::new().analyze(&project, dir.path());

    assert_eq!(report.health_score(), 100);
}

#[test]
fn health_score_drops_with_duplicate_versions() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "Cargo.toml",
        r#"
            [package]
            name = "sample"
            version = "0.1.0"

            [dependencies]
            serde = "1"
        "#,
    );
    write(
        &dir,
        "Cargo.lock",
        r#"
            [[package]]
            name = "syn"
            version = "1.0.0"

            [[package]]
            name = "syn"
            version = "2.0.0"
        "#,
    );

    let project = ProjectDetector::new().detect(dir.path()).unwrap();
    let report = Analyzer::new().analyze(&project, dir.path());

    assert!(report.health_score() < 100);
}
