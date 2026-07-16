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

#[test]
fn detects_cargo_workspace() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/*\"]\n",
    );

    let project = ProjectDetector::new().detect(dir.path()).unwrap();

    assert!(project.is_workspace);
    assert_eq!(project.config_file, "Cargo.toml");
}

#[test]
fn detects_vite_only_when_no_other_framework_present() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "package.json",
        r#"{"name": "sample", "devDependencies": {"vite": "^5.0.0"}}"#,
    );

    let project = ProjectDetector::new().detect(dir.path()).unwrap();

    assert_eq!(project.framework, Framework::Vite);
}

#[test]
fn react_takes_priority_over_vite() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "package.json",
        r#"{"name": "sample", "dependencies": {"react": "^18.0.0"}, "devDependencies": {"vite": "^5.0.0"}}"#,
    );

    let project = ProjectDetector::new().detect(dir.path()).unwrap();

    assert_eq!(project.framework, Framework::React);
}

#[test]
fn detects_python_virtualenv() {
    let dir = TempDir::new().unwrap();
    write(&dir, "requirements.txt", "flask==2.3.0\n");
    fs::create_dir_all(dir.path().join(".venv")).unwrap();

    let project = ProjectDetector::new().detect(dir.path()).unwrap();

    assert!(project.has_virtualenv);
}

#[test]
fn custom_ignore_list_excludes_extra_directories_from_file_count() {
    let without_ignore_dir = TempDir::new().unwrap();
    write(
        &without_ignore_dir,
        "Cargo.toml",
        "[package]\nname = \"sample\"\n",
    );
    write(&without_ignore_dir, "src/main.rs", "fn main() {}");
    write(&without_ignore_dir, "vendor/some_file.rs", "// vendored");
    let without_ignore = ProjectDetector::new()
        .detect(without_ignore_dir.path())
        .unwrap();
    assert_eq!(without_ignore.file_count, 3);

    let with_ignore_dir = TempDir::new().unwrap();
    write(
        &with_ignore_dir,
        "Cargo.toml",
        "[package]\nname = \"sample\"\n",
    );
    write(&with_ignore_dir, "src/main.rs", "fn main() {}");
    write(&with_ignore_dir, "vendor/some_file.rs", "// vendored");
    write(
        &with_ignore_dir,
        "blink.toml",
        "[project]\nname = \"sample\"\nignore = [\"vendor\"]\n",
    );
    let with_ignore = ProjectDetector::new()
        .detect(with_ignore_dir.path())
        .unwrap();

    // Cargo.toml, src/main.rs, blink.toml — vendor/some_file.rs excluded.
    assert_eq!(with_ignore.file_count, 3);
}
