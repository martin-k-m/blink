use std::fs;

use tempfile::TempDir;

use crate::{BlinkConfig, Framework, Language, PackageManager, ProjectDetector};

fn write(dir: &TempDir, name: &str, contents: &str) {
    let path = dir.path().join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[test]
fn detects_rust_cargo_project() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "Cargo.toml",
        r#"
            [package]
            name = "sample-crate"
            version = "0.1.0"

            [dependencies]
            serde = "1"
            tokio = { version = "1", features = ["full"] }
        "#,
    );
    write(&dir, "src/main.rs", "fn main() {}");

    let project = ProjectDetector::new().detect(dir.path()).unwrap();

    assert_eq!(project.name, "sample-crate");
    assert_eq!(project.language, Language::Rust);
    assert_eq!(project.framework, Framework::Cargo);
    assert_eq!(project.package_manager, PackageManager::Cargo);
    assert_eq!(project.dependency_count(), 2);
}

#[test]
fn detects_react_typescript_project() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "package.json",
        r#"{
            "name": "sample-app",
            "dependencies": { "react": "^18.0.0", "react-dom": "^18.0.0" },
            "devDependencies": { "typescript": "^5.0.0" }
        }"#,
    );
    write(&dir, "tsconfig.json", "{}");
    write(&dir, "src/App.tsx", "export default function App() {}");

    let project = ProjectDetector::new().detect(dir.path()).unwrap();

    assert_eq!(project.name, "sample-app");
    assert_eq!(project.language, Language::TypeScript);
    assert_eq!(project.framework, Framework::React);
    assert_eq!(project.dependency_count(), 3);
}

#[test]
fn detects_package_manager_from_lockfile() {
    let dir = TempDir::new().unwrap();
    write(&dir, "package.json", r#"{"name": "sample"}"#);
    write(&dir, "pnpm-lock.yaml", "lockfileVersion: 6.0");

    let project = ProjectDetector::new().detect(dir.path()).unwrap();

    assert_eq!(project.package_manager, PackageManager::Pnpm);
}

#[test]
fn returns_error_for_unknown_project() {
    let dir = TempDir::new().unwrap();

    let result = ProjectDetector::new().detect(dir.path());

    assert!(result.is_err());
}

#[test]
fn config_round_trips_through_toml() {
    let dir = TempDir::new().unwrap();
    let config = BlinkConfig::new("my-app");

    config.write(dir.path()).unwrap();
    let loaded = BlinkConfig::load(dir.path()).unwrap();

    assert_eq!(config, loaded);
    assert_eq!(loaded.server.port, 3000);
    assert!(loaded.optimization.cache);
}

#[test]
fn config_exists_reports_presence() {
    let dir = TempDir::new().unwrap();
    assert!(!BlinkConfig::exists(dir.path()));

    BlinkConfig::new("my-app").write(dir.path()).unwrap();

    assert!(BlinkConfig::exists(dir.path()));
}
