use std::path::PathBuf;

use blink_core::{Framework, Language, PackageManager, ProjectDetector};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
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
