use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

fn blink() -> Command {
    Command::cargo_bin("blink").expect("blink binary should build")
}

#[test]
fn scan_reports_detected_react_project() {
    blink()
        .arg("scan")
        .arg(fixture("react-app"))
        .assert()
        .success()
        .stdout(predicate::str::contains("React"))
        .stdout(predicate::str::contains("TypeScript"))
        .stdout(predicate::str::contains("Scan completed"));
}

#[test]
fn scan_reports_detected_rust_project() {
    blink()
        .arg("scan")
        .arg(fixture("rust-crate"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust"))
        .stdout(predicate::str::contains("Cargo"));
}

#[test]
fn scan_fails_on_unrecognized_directory() {
    let dir = tempfile::TempDir::new().unwrap();

    blink().arg("scan").arg(dir.path()).assert().failure();
}

#[test]
fn analyze_reports_dependency_health() {
    blink()
        .arg("analyze")
        .arg(fixture("react-app"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Blink Analysis Report"))
        .stdout(predicate::str::contains("Healthy packages"));
}

#[test]
fn init_creates_blink_toml() {
    let dir = tempfile::TempDir::new().unwrap();
    let project_dir = dir.path().join("my-app");

    blink()
        .arg("init")
        .arg(&project_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Project initialized"));

    let config = std::fs::read_to_string(project_dir.join("blink.toml")).unwrap();
    assert!(config.contains("name = \"my-app\""));
    assert!(config.contains("port = 3000"));
}

#[test]
fn build_creates_and_reuses_cache() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join("a.txt"), "hello").unwrap();

    blink()
        .arg("build")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("no previous cache"));

    blink()
        .arg("build")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("unchanged"));

    assert!(dir.path().join(".blink").join("cache.json").is_file());
}
