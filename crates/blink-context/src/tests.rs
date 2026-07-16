//! End-to-end tests that build a real graph from a temporary project on disk.

use std::fs;
use std::path::Path;

use tempfile::TempDir;

use crate::build::area_of;
use crate::ContextGraph;

fn write(root: &Path, rel: &str, contents: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

/// A small TypeScript app: `app.ts` imports an API area and a DB area.
fn scaffold_ts_app() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write(
        root,
        "package.json",
        r#"{ "name": "example-app", "dependencies": { "left-pad": "^1.0.0" } }"#,
    );
    // A tsconfig makes detection unambiguously TypeScript (not a source file
    // itself, so it doesn't change the source-file count).
    write(root, "tsconfig.json", "{}\n");
    write(
        root,
        "src/app.ts",
        "import { router } from './api';\nimport { connect } from './db/conn';\nexport const start = () => router;\n",
    );
    write(
        root,
        "src/api/index.ts",
        "/**\n * HTTP routes for the app.\n */\nexport function router() {}\n",
    );
    write(
        root,
        "src/db/conn.ts",
        "export function connect() {}\nexport interface Conn {}\n",
    );
    dir
}

#[test]
fn area_grouping_rule() {
    assert_eq!(area_of("README.md"), "(root)");
    assert_eq!(area_of("src/main.rs"), "src");
    assert_eq!(area_of("src/auth/login.rs"), "src/auth");
    assert_eq!(area_of("crates/blink-core/src/lib.rs"), "crates/blink-core");
    assert_eq!(area_of("docs/cli.md"), "docs");
}

#[test]
fn builds_graph_with_identity_and_stats() {
    let dir = scaffold_ts_app();
    let graph = ContextGraph::build(dir.path()).unwrap();

    assert_eq!(graph.project.name, "example-app");
    assert_eq!(graph.project.language, "TypeScript");
    assert_eq!(graph.project.package_manager, "npm");

    // Three .ts source files were written.
    assert_eq!(graph.stats.source_files, 3);
    assert!(graph.stats.symbols >= 3, "symbols: {}", graph.stats.symbols);

    // The declared dependency shows up.
    assert!(graph.dependencies.iter().any(|d| d.name == "left-pad"));
}

#[test]
fn groups_files_into_areas() {
    let dir = scaffold_ts_app();
    let graph = ContextGraph::build(dir.path()).unwrap();
    let areas: Vec<&str> = graph.areas.iter().map(|a| a.path.as_str()).collect();
    assert!(areas.contains(&"src"), "areas: {areas:?}");
    assert!(areas.contains(&"src/api"), "areas: {areas:?}");
    assert!(areas.contains(&"src/db"), "areas: {areas:?}");
}

#[test]
fn resolves_references_between_files() {
    let dir = scaffold_ts_app();
    let graph = ContextGraph::build(dir.path()).unwrap();

    let deps = graph.dependencies_of("src/app.ts");
    assert!(deps.contains(&"src/api/index.ts"), "deps: {deps:?}");
    assert!(deps.contains(&"src/db/conn.ts"), "deps: {deps:?}");

    let dependents = graph.dependents_of("src/api/index.ts");
    assert_eq!(dependents, vec!["src/app.ts"]);
}

#[test]
fn area_edges_cross_areas_only() {
    let dir = scaffold_ts_app();
    let graph = ContextGraph::build(dir.path()).unwrap();
    let edges = graph.area_edges();
    // src -> src/api and src -> src/db (app.ts lives in the `src` area).
    assert!(
        edges.iter().any(|e| e.from == "src" && e.to == "src/api"),
        "edges: {edges:?}"
    );
    assert!(
        edges.iter().any(|e| e.from == "src" && e.to == "src/db"),
        "edges: {edges:?}"
    );
}

#[test]
fn explain_uses_only_real_signals() {
    let dir = scaffold_ts_app();
    let graph = ContextGraph::build(dir.path()).unwrap();

    let ex = graph.explain("src/api/index.ts").unwrap();
    assert_eq!(ex.doc.as_deref(), Some("HTTP routes for the app."));
    assert!(ex.symbols.iter().any(|s| s.name == "router"));
    assert_eq!(ex.dependents, vec!["src/app.ts"]);

    let app = graph.explain("src/app.ts").unwrap();
    assert!(app.references.contains(&"src/api/index.ts".to_string()));
}

#[test]
fn rust_workspace_cross_crate_edges() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write(
        root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/a\", \"crates/b\"]\n",
    );
    write(root, "crates/a/Cargo.toml", "[package]\nname = \"my-a\"\n");
    write(root, "crates/a/src/lib.rs", "pub fn helper() {}\n");
    write(root, "crates/b/Cargo.toml", "[package]\nname = \"my-b\"\n");
    write(
        root,
        "crates/b/src/lib.rs",
        "use my_a::helper;\npub fn go() { helper() }\n",
    );

    let graph = ContextGraph::build(root).unwrap();

    // `use my_a::…` in crate b resolves to crate a's lib root.
    let deps = graph.dependencies_of("crates/b/src/lib.rs");
    assert!(deps.contains(&"crates/a/src/lib.rs"), "deps: {deps:?}");

    // And it surfaces as a cross-area edge in the map.
    let edges = graph.area_edges();
    assert!(
        edges
            .iter()
            .any(|e| e.from == "crates/b" && e.to == "crates/a"),
        "edges: {edges:?}"
    );
}

#[test]
fn include_filter_limits_coverage() {
    let dir = scaffold_ts_app();
    let root = dir.path();
    // Restrict the context graph to `src/db` only.
    write(
        root,
        ".bnk",
        "[project]\nname = \"example-app\"\n\n[context]\ninclude = [\"src/db\"]\n",
    );
    let graph = ContextGraph::build(root).unwrap();
    assert!(graph.files.iter().all(|f| f.path.starts_with("src/db")));
    assert_eq!(graph.config.include, vec!["src/db".to_string()]);
}
