use std::fs;
use std::path::Path;

use tempfile::TempDir;

use blink_analyzer::Analyzer;
use blink_core::ProjectDetector;
use blink_index::Index;

use crate::{clean, config_audit, doctor, duplicates, env, filesystem, optimize, tasks};

fn write(dir: &Path, rel: &str, contents: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[test]
fn discovers_tasks_from_multiple_sources_with_config_precedence() {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "package.json",
        r#"{"name":"x","scripts":{"dev":"vite","build":"vite build"}}"#,
    );
    write(
        dir.path(),
        "Makefile",
        "deploy:\n\techo shipping\n.PHONY: deploy\n",
    );
    write(
        dir.path(),
        "blink.toml",
        "[project]\nname = \"x\"\n[commands]\ndev = \"custom dev\"\n",
    );

    let tasks = tasks::discover(dir.path());
    let dev = tasks.iter().find(|t| t.name == "dev").unwrap();
    // Config wins over the package.json script of the same name.
    assert_eq!(dev.command, "custom dev");
    assert_eq!(dev.source, tasks::TaskSource::BlinkConfig);

    assert!(tasks
        .iter()
        .any(|t| t.name == "build" && t.source == tasks::TaskSource::PackageJson));
    assert!(tasks
        .iter()
        .any(|t| t.name == "deploy" && t.command == "make deploy"));

    let found = tasks::find(dir.path(), "build").unwrap();
    assert_eq!(found.command, "vite build");
    assert!(tasks::find(dir.path(), "nope").is_none());
}

#[test]
fn clean_plan_finds_artifact_dirs_and_marks_heavy() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "target/debug/app", "binary-ish");
    write(dir.path(), "dist/bundle.js", "console.log(1)");
    write(dir.path(), "src/main.rs", "fn main() {}");

    let plan = clean::plan(dir.path());
    let names: Vec<&str> = plan.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"target"));
    assert!(names.contains(&"dist"));
    assert!(!names.contains(&"src"));

    let target = plan.iter().find(|t| t.name == "target").unwrap();
    assert!(target.heavy);
    assert!(target.bytes > 0);
    let dist = plan.iter().find(|t| t.name == "dist").unwrap();
    assert!(!dist.heavy);
    assert_eq!(clean::total_bytes(&plan), target.bytes + dist.bytes);
}

#[test]
fn env_compare_reports_missing_and_unused_names_only() {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        ".env.example",
        "# config\nDATABASE_URL=\nexport API_KEY=\nPORT=3000\n",
    );
    write(
        dir.path(),
        ".env",
        "DATABASE_URL=postgres://secret\nEXTRA=1\n",
    );

    let report = env::compare(dir.path());
    assert!(report.has_env && report.has_example);
    assert_eq!(report.configured, vec!["DATABASE_URL"]);
    assert!(report.missing.contains(&"API_KEY".to_string()));
    assert!(report.missing.contains(&"PORT".to_string()));
    assert_eq!(report.unused, vec!["EXTRA"]);
    assert!(!report.is_complete());
}

#[test]
fn doctor_flags_missing_required_runtime_but_not_optional() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "Cargo.toml", "[package]\nname = \"x\"\n");
    let project = ProjectDetector::new().detect(dir.path()).unwrap();

    let report = doctor::diagnose(&project, dir.path());
    // cargo/rustc are required checks; git is optional. Whether they're present
    // depends on the host, but the *shape* must hold: required checks exist.
    assert!(report
        .checks
        .iter()
        .any(|c| c.name == "cargo" && c.required));
    assert!(report.checks.iter().any(|c| c.name == "git" && !c.required));
}

#[test]
fn doctor_reports_missing_env_var_names() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "Cargo.toml", "[package]\nname = \"x\"\n");
    write(dir.path(), ".env.example", "SECRET_TOKEN=\n");
    // No .env at all → SECRET_TOKEN is missing.
    let project = ProjectDetector::new().detect(dir.path()).unwrap();
    let report = doctor::diagnose(&project, dir.path());
    assert!(report.checks.iter().any(|c| c.name == "env: SECRET_TOKEN"));
}

#[test]
fn duplicate_files_detected_by_content() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "a.txt", "identical contents here");
    write(dir.path(), "nested/b.txt", "identical contents here");
    write(dir.path(), "c.txt", "unique");
    write(dir.path(), "empty1", "");
    write(dir.path(), "empty2", "");

    let (index, _) = Index::build(dir.path()).unwrap();
    let groups = duplicates::find(&index);

    // Only the two identical non-empty files form a group; empties are ignored.
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].paths, vec!["a.txt", "nested/b.txt"]);
    assert_eq!(
        groups[0].wasted_bytes(),
        "identical contents here".len() as u64
    );
    assert_eq!(duplicates::total_wasted(&groups), groups[0].wasted_bytes());
}

#[test]
fn config_audit_detects_present_and_missing() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "README.md", "# hi");
    write(dir.path(), ".github/workflows/ci.yml", "name: ci");

    let items = config_audit::audit(dir.path());
    let by_name = |n: &str| items.iter().find(|i| i.name == n).unwrap();
    assert!(by_name("README").present);
    assert!(by_name("CI configuration").present);
    assert!(!by_name("LICENSE").present);
    assert!(!by_name(".gitignore").present);
    assert!(by_name(".gitignore").required);
}

#[test]
fn filesystem_report_separates_ignored_weight() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "src/main.rs", "fn main() {}");
    write(dir.path(), "target/big", &"x".repeat(5000));

    let report = filesystem::analyze(dir.path());
    assert!(report.total_bytes >= 5000);
    assert!(report.ignored_bytes >= 5000);
    assert!(report.source_bytes() < report.ignored_bytes);
    // target should be the largest entry and marked ignored.
    let biggest = &report.entries[0];
    assert_eq!(biggest.name, "target");
    assert!(biggest.ignored);
}

#[test]
fn optimize_flags_real_conditions_and_scores() {
    let dir = TempDir::new().unwrap();
    // A Node project with an unused runtime dependency and no
    // README/gitignore/tests. (Rust's analyzer intentionally treats Cargo deps
    // as framework markers, so unused detection there is suppressed by design —
    // a JS project is where the dependency rule actually fires.)
    write(
        dir.path(),
        "package.json",
        r#"{"name":"x","dependencies":{"left-pad":"^1.0.0"}}"#,
    );
    write(dir.path(), "src/index.js", "console.log('hi');\n");

    let project = ProjectDetector::new().detect(dir.path()).unwrap();
    let report = Analyzer::new().analyze(&project, dir.path());
    let (index, _) = Index::build(dir.path()).unwrap();

    let opt = optimize::optimize(&project, &report, &index, dir.path());

    // Dependencies should warn (left-pad isn't referenced in source).
    let deps = opt
        .checks
        .iter()
        .find(|c| c.category == "Dependencies")
        .unwrap();
    assert_eq!(deps.status, optimize::OptStatus::Warn);
    // Tests + Documentation + Configuration should also warn here.
    assert!(opt.warnings() >= 3);
    assert!(opt.score < 100);
    // Score follows the documented rule exactly.
    let expected = 100u8.saturating_sub(optimize::WARN_PENALTY * opt.warnings() as u8);
    assert_eq!(opt.score, expected);
    assert!(opt.suggestions.iter().any(|s| s.contains("left-pad")));
}
