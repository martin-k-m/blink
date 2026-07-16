use std::path::PathBuf;

use blink_core::{Framework, Language, PackageManager, ProjectDetector};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

fn tests_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn detects_rust_cargo_example() {
    let project = ProjectDetector::new()
        .detect(&fixture("rust-crate"))
        .expect("rust-crate example should be detected");

    assert_eq!(project.name, "rust-crate-example");
    assert_eq!(project.language, Language::Rust);
    assert_eq!(project.framework, Framework::Cargo);
    assert_eq!(project.package_manager, PackageManager::Cargo);
}

#[test]
fn rust_example_declares_expected_dependencies() {
    let project = ProjectDetector::new()
        .detect(&fixture("rust-crate"))
        .unwrap();

    let names: Vec<&str> = project
        .dependencies
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(names.contains(&"serde"));
    assert!(names.contains(&"serde_json"));
}

#[test]
fn detecting_a_directory_with_no_manifest_fails() {
    let dir = tempfile::TempDir::new().unwrap();

    let result = ProjectDetector::new().detect(dir.path());

    assert!(result.is_err());
}

#[test]
fn rust_fixture_reports_duplicate_versions_from_its_lockfile() {
    let root = tests_fixture("rust_project");
    let project = ProjectDetector::new().detect(&root).unwrap();
    let report = blink_analyzer::Analyzer::new().analyze(&project, &root);

    assert_eq!(report.duplicates.len(), 1);
    assert_eq!(report.duplicates[0].name, "syn");
    assert_eq!(report.duplicates[0].versions, vec!["1.0.109", "2.0.48"]);
}

#[test]
fn rust_fixture_dependency_counts_include_transitive_from_lockfile() {
    let root = tests_fixture("rust_project");
    let project = ProjectDetector::new().detect(&root).unwrap();
    let report = blink_analyzer::Analyzer::new().analyze(&project, &root);

    assert_eq!(report.dependency_counts.direct, 1);
    // 4 total locked packages (fixture crate + serde + syn x2) minus 1 direct dependency.
    assert_eq!(report.dependency_counts.transitive, Some(3));
}
