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
fn detects_react_typescript_example() {
    let project = ProjectDetector::new()
        .detect(&fixture("react-app"))
        .expect("react-app example should be detected");

    assert_eq!(project.name, "react-app-example");
    assert_eq!(project.language, Language::TypeScript);
    assert_eq!(project.framework, Framework::React);
    assert_eq!(project.package_manager, PackageManager::Npm);
}

#[test]
fn react_example_declares_expected_dependencies() {
    let project = ProjectDetector::new()
        .detect(&fixture("react-app"))
        .unwrap();

    let names: Vec<&str> = project
        .dependencies
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(names.contains(&"react"));
    assert!(names.contains(&"react-dom"));
    assert!(names.contains(&"typescript"));

    let react = project
        .dependencies
        .iter()
        .find(|d| d.name == "react")
        .unwrap();
    assert!(!react.dev);
    let typescript = project
        .dependencies
        .iter()
        .find(|d| d.name == "typescript")
        .unwrap();
    assert!(typescript.dev);
}

#[test]
fn react_example_has_no_unused_runtime_dependencies() {
    let root = fixture("react-app");
    let project = ProjectDetector::new().detect(&root).unwrap();
    let report = blink_analyzer::Analyzer::new().analyze(&project, &root);

    assert!(
        report.unused.is_empty(),
        "expected no unused dependencies, found {:?}",
        report.unused
    );
}

#[test]
fn react_fixture_flags_its_deliberately_unused_dependency() {
    let root = tests_fixture("react_project");
    let project = ProjectDetector::new().detect(&root).unwrap();
    let report = blink_analyzer::Analyzer::new().analyze(&project, &root);

    assert_eq!(report.unused, vec!["moment".to_string()]);
}
