use std::path::PathBuf;

use blink_core::{Framework, Language, PackageManager, ProjectDetector};

fn tests_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn detects_python_project() {
    let project = ProjectDetector::new()
        .detect(&tests_fixture("python_project"))
        .expect("python_project fixture should be detected");

    assert_eq!(project.language, Language::Python);
    assert_eq!(project.framework, Framework::None);
    assert_eq!(project.package_manager, PackageManager::Pip);
}

#[test]
fn python_project_declares_expected_dependencies() {
    let project = ProjectDetector::new()
        .detect(&tests_fixture("python_project"))
        .unwrap();

    let names: Vec<&str> = project
        .dependencies
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(names.contains(&"flask"));
    assert!(names.contains(&"requests"));

    let flask = project
        .dependencies
        .iter()
        .find(|d| d.name == "flask")
        .unwrap();
    assert_eq!(flask.version, "2.3.0");
}

#[test]
fn python_project_flags_its_deliberately_unused_dependency() {
    let root = tests_fixture("python_project");
    let project = ProjectDetector::new().detect(&root).unwrap();
    let report = blink_analyzer::Analyzer::new().analyze(&project, &root);

    assert_eq!(report.unused, vec!["requests".to_string()]);
}
