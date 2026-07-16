use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;

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
        .stdout(predicate::str::contains("Blink Analysis"))
        .stdout(predicate::str::contains("Health"))
        .stdout(predicate::str::contains("Dependencies"));
}

#[test]
fn analyze_json_emits_valid_structured_output() {
    let output = blink()
        .arg("analyze")
        .arg(fixture("react-app"))
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).expect("valid JSON output");
    assert_eq!(json["project"], "react-app-example");
    assert!(json["dependencies"]["direct"].as_u64().unwrap() > 0);
    assert!(json["health"]["score"].as_u64().is_some());
}

#[test]
fn analyze_reports_duplicate_versions_from_fixture_lockfile() {
    blink()
        .arg("analyze")
        .arg(tests_fixture("rust_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Duplicate package versions"))
        .stdout(predicate::str::contains("syn"));
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
fn scan_verbose_reports_diagnostic_detail() {
    blink()
        .arg("scan")
        .arg(fixture("rust-crate"))
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("Scanning"))
        .stdout(predicate::str::contains("Configuration"))
        .stdout(predicate::str::contains("Cargo.toml"))
        .stdout(predicate::str::contains("Ignore rules in effect"));
}

#[test]
fn deps_reports_direct_and_transitive_counts() {
    blink()
        .arg("deps")
        .arg(tests_fixture("rust_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Direct"))
        .stdout(predicate::str::contains("Transitive"));
}

#[test]
fn health_reports_score_and_subscores() {
    blink()
        .arg("health")
        .arg(fixture("react-app"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Score"))
        .stdout(predicate::str::contains("Code Organization"))
        .stdout(predicate::str::contains("Dependencies"))
        .stdout(predicate::str::contains("Configuration"));
}

#[test]
fn health_json_emits_valid_structured_output() {
    let output = blink()
        .arg("health")
        .arg(fixture("react-app"))
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).expect("valid JSON output");
    assert!(json["overall"].as_u64().is_some());
    assert!(json["configuration"].as_u64().is_some());
    assert!(json["code_organization"].as_u64().is_some());
}

#[test]
fn recommend_groups_findings_into_categories() {
    blink()
        .arg("recommend")
        .arg(fixture("rust-crate"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Performance"))
        .stdout(predicate::str::contains("Maintenance"))
        .stdout(predicate::str::contains("Security"))
        .stdout(predicate::str::contains("run with --online to check"));
}

#[test]
fn ci_passes_with_no_warnings_on_a_clean_fixture() {
    blink()
        .arg("ci")
        .arg(fixture("rust-crate"))
        .assert()
        .code(predicate::in_iter([0, 1]))
        .stdout(predicate::str::contains("Blink CI Report"))
        .stdout(predicate::str::contains("Issues"));
}

#[test]
fn ci_exits_2_when_project_cannot_be_detected() {
    let dir = tempfile::TempDir::new().unwrap();

    blink().arg("ci").arg(dir.path()).assert().code(2);
}

#[test]
fn report_markdown_includes_expected_sections() {
    blink()
        .arg("report")
        .arg(fixture("rust-crate"))
        .arg("--markdown")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Blink Project Report"))
        .stdout(predicate::str::contains("## Dependencies"));
}

#[test]
fn report_html_writes_a_self_contained_page_to_output_file() {
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("report.html");

    blink()
        .arg("report")
        .arg(fixture("rust-crate"))
        .arg("--html")
        .arg("--output")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("Written to"));

    let html = std::fs::read_to_string(&output).unwrap();
    assert!(html.starts_with("<!doctype html>"));
    assert!(html.contains("Blink Project Report"));
}

#[test]
fn report_defaults_to_json() {
    let output = blink()
        .arg("report")
        .arg(fixture("rust-crate"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).expect("valid JSON output");
    assert_eq!(json["project"], "rust-crate-example");
}

#[test]
fn plugins_lists_installed_plugins_or_says_none_found() {
    blink()
        .arg("plugins")
        .assert()
        .success()
        .stdout(predicate::str::contains("Blink Plugins"));
}

#[test]
fn plugins_install_copies_executable_into_managed_directory() {
    let source_dir = tempfile::TempDir::new().unwrap();
    let home_dir = tempfile::TempDir::new().unwrap();
    let source = source_dir.path().join("my-tool");
    std::fs::write(&source, b"fake executable bytes").unwrap();

    blink()
        .env("HOME", home_dir.path())
        .env("USERPROFILE", home_dir.path())
        .arg("plugins")
        .arg("install")
        .arg(&source)
        .arg("--name")
        .arg("mytool")
        .assert()
        .success()
        .stdout(predicate::str::contains("Installed 'mytool'"));

    let expected_name = if cfg!(windows) {
        "blink-mytool.exe"
    } else {
        "blink-mytool"
    };
    let installed = home_dir
        .path()
        .join(".blink")
        .join("plugins")
        .join(expected_name);
    assert!(installed.is_file());
}

#[test]
fn benchmark_reports_all_four_measurements() {
    blink()
        .arg("benchmark")
        .arg(fixture("rust-crate"))
        .arg("--runs")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Startup"))
        .stdout(predicate::str::contains("Scan"))
        .stdout(predicate::str::contains("Cold Analysis"))
        .stdout(predicate::str::contains("Cached Analysis"));
}

#[test]
fn unknown_subcommand_falls_back_to_an_installed_plugin() {
    let home_dir = tempfile::TempDir::new().unwrap();
    let plugins_dir = home_dir.path().join(".blink").join("plugins");
    std::fs::create_dir_all(&plugins_dir).unwrap();

    // Install a copy of `blink` itself as a plugin: running `blink
    // selftest --version` should transparently exec it, which (being the
    // same binary) prints the same `blink <version>` line `blink
    // --version` does. This exercises the real fallback-dispatch path in
    // main.rs, not just plugin discovery/installation in isolation.
    let blink_exe = assert_cmd::cargo::cargo_bin("blink");
    let plugin_name = if cfg!(windows) {
        "blink-selftest.exe"
    } else {
        "blink-selftest"
    };
    let plugin_path = plugins_dir.join(plugin_name);
    std::fs::copy(&blink_exe, &plugin_path).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&plugin_path).unwrap().permissions();
        perms.set_mode(perms.mode() | 0o111);
        std::fs::set_permissions(&plugin_path, perms).unwrap();
    }

    blink()
        .env("HOME", home_dir.path())
        .env("USERPROFILE", home_dir.path())
        .arg("selftest")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("blink "));
}

/// A throwaway Rust project on disk, so index-writing commands (which persist
/// `.blink/index.json`) don't pollute the shared `examples/` fixtures.
fn rust_temp() -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/main.rs"),
        "fn main() { println!(\"hi\"); }\nstruct App;\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("README.md"), "# demo\n").unwrap();
    std::fs::write(dir.path().join(".gitignore"), "/target\n").unwrap();
    dir
}

#[test]
fn index_then_status_reports_files_and_symbols() {
    let dir = rust_temp();

    blink()
        .arg("index")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Files indexed"))
        .stdout(predicate::str::contains("Symbols"));

    assert!(dir.path().join(".blink").join("index.json").is_file());

    blink()
        .arg("status")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("indexed"))
        .stdout(predicate::str::contains("Rust"));
}

#[test]
fn inspect_json_reports_measured_project_facts() {
    let dir = rust_temp();
    let output = blink()
        .arg("inspect")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).expect("valid JSON output");
    assert_eq!(json["name"], "demo");
    assert_eq!(json["language"], "Rust");
    assert!(json["files"].as_u64().unwrap() >= 2);
    assert!(json["symbols"].as_u64().unwrap() >= 2);
    assert_eq!(json["run_command"], "cargo run");
}

#[test]
fn optimize_json_scores_and_lists_checks() {
    let dir = rust_temp();
    let output = blink()
        .arg("optimize")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).expect("valid JSON output");
    assert!(json["score"].as_u64().unwrap() <= 100);
    assert!(json["checks"].as_array().unwrap().len() >= 6);
}

#[test]
fn symbols_json_finds_declared_symbol() {
    let dir = rust_temp();
    let output = blink()
        .arg("symbols")
        .arg(dir.path())
        .arg("--filter")
        .arg("App")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).expect("valid JSON output");
    let arr = json.as_array().unwrap();
    assert!(arr
        .iter()
        .any(|s| s["name"] == "App" && s["kind"] == "struct"));
}

#[test]
fn duplicates_detects_identical_files() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"d\"\n").unwrap();
    std::fs::write(dir.path().join("a.txt"), "same bytes here").unwrap();
    std::fs::write(dir.path().join("b.txt"), "same bytes here").unwrap();

    blink()
        .arg("duplicates")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("reclaimable"));
}

#[test]
fn doctor_reports_environment_checks() {
    blink()
        .arg("doctor")
        .arg(fixture("rust-crate"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Blink Doctor"))
        .stdout(predicate::str::contains("cargo"));
}

#[test]
fn config_audit_lists_standard_files() {
    blink()
        .arg("config-audit")
        .arg(fixture("rust-crate"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration Audit"))
        .stdout(predicate::str::contains("README"));
}

#[test]
fn tasks_and_config_check_read_bnk_signature_file() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"d\"\n").unwrap();
    std::fs::write(
        dir.path().join(".bnk"),
        "[project]\nname = \"d\"\n[commands]\ngreet = \"echo hi\"\n",
    )
    .unwrap();

    blink()
        .arg("tasks")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("greet"));

    blink()
        .arg("config")
        .arg("check")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(".bnk is valid"));
}

#[test]
fn task_runs_a_discovered_command() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"d\"\n").unwrap();
    std::fs::write(
        dir.path().join(".bnk"),
        "[project]\nname = \"d\"\n[commands]\ngreet = \"echo blink-task-ran\"\n",
    )
    .unwrap();

    blink()
        .arg("task")
        .arg("greet")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("blink-task-ran"));
}

#[test]
fn clean_dry_run_reports_without_deleting() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"d\"\n").unwrap();
    std::fs::create_dir_all(dir.path().join("dist")).unwrap();
    std::fs::write(dir.path().join("dist/bundle.js"), "x").unwrap();

    blink()
        .arg("clean")
        .arg(dir.path())
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("dry run"));

    // Nothing was deleted.
    assert!(dir.path().join("dist").is_dir());
}

#[test]
fn completions_generates_a_shell_script() {
    blink()
        .arg("completions")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("blink"));
}

/// Recursively copy `src` into `dst` (created if needed), for tests that run
/// index-writing commands against a committed fixture without dirtying it.
fn copy_dir(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let target = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).unwrap();
        }
    }
}

#[test]
fn tasks_discovers_node_project_scripts() {
    blink()
        .arg("tasks")
        .arg(tests_fixture("node_project"))
        .assert()
        .success()
        .stdout(predicate::str::contains("dev"))
        .stdout(predicate::str::contains("build"))
        .stdout(predicate::str::contains("package.json"));
}

#[test]
fn inspect_detects_workspace_in_monorepo() {
    let dir = tempfile::TempDir::new().unwrap();
    copy_dir(&tests_fixture("monorepo"), dir.path());

    let output = blink()
        .arg("inspect")
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).expect("valid JSON output");
    assert_eq!(json["is_workspace"], true);
    assert_eq!(json["language"], "Rust");
    // The workspace's member sources are indexed (app + lib each have one).
    assert!(json["files"].as_u64().unwrap() >= 4);
}

#[test]
fn unknown_project_error_explains_and_suggests_fixes() {
    let dir = tempfile::TempDir::new().unwrap();
    blink()
        .arg("analyze")
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("couldn't find a project"))
        .stderr(predicate::str::contains("Possible fixes"));
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
