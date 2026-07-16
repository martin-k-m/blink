use std::path::PathBuf;

use blink_context::{
    Area, CommandNode, ConfigInfo, ContextGraph, DependencyNode, FileNode, ProjectInfo, Stats,
    SymbolRef,
};

use super::*;

fn sym(name: &str, kind: &str, line: usize) -> SymbolRef {
    SymbolRef {
        name: name.to_string(),
        kind: kind.to_string(),
        line,
    }
}

fn graph() -> ContextGraph {
    ContextGraph {
        project: ProjectInfo {
            name: "demo".into(),
            language: "TypeScript".into(),
            framework: None,
            package_manager: "npm".into(),
            is_workspace: false,
        },
        stats: Stats::default(),
        config: ConfigInfo {
            present: false,
            file: None,
            context_enabled: true,
            include: Vec::new(),
        },
        areas: vec![
            Area {
                path: "src/api".into(),
                files: 3,
                lines: 120,
                symbols: 8,
                size_bytes: 4000,
                languages: vec!["TypeScript".into()],
            },
            Area {
                path: "src/db".into(),
                files: 2,
                lines: 80,
                symbols: 4,
                size_bytes: 2000,
                languages: vec!["TypeScript".into()],
            },
        ],
        dependencies: vec![DependencyNode {
            name: "express".into(),
            version: "^4.0.0".into(),
            dev: false,
        }],
        commands: vec![CommandNode {
            name: "dev".into(),
            command: "vite".into(),
            source: "package.json".into(),
        }],
        files: vec![
            FileNode {
                path: "src/api/routes.ts".into(),
                area: "src/api".into(),
                lang: Some("TypeScript".into()),
                lines: 60,
                size_bytes: 2000,
                symbols: vec![sym("registerRoutes", "fn", 10), sym("apiRouter", "fn", 20)],
            },
            FileNode {
                path: "src/db/connection.ts".into(),
                area: "src/db".into(),
                lang: Some("TypeScript".into()),
                lines: 40,
                size_bytes: 1500,
                symbols: vec![sym("connect", "fn", 5)],
            },
        ],
        references: Vec::new(),
        root: PathBuf::new(),
    }
}

#[test]
fn strips_question_words() {
    let r = query(&graph(), "where are the API routes", 10);
    assert_eq!(r.terms, vec!["api".to_string(), "routes".to_string()]);
}

#[test]
fn finds_area_and_file_for_api_routes() {
    let r = query(&graph(), "api routes", 10);
    assert!(r.areas.iter().any(|m| m.name == "src/api"));
    assert!(r.files.iter().any(|m| m.name == "src/api/routes.ts"));
    // The routes file should out-rank anything in db.
    assert_eq!(r.files.first().unwrap().name, "src/api/routes.ts");
}

#[test]
fn matches_symbols_by_camel_split() {
    let r = query(&graph(), "register", 10);
    assert!(r.symbols.iter().any(|s| s.name == "registerRoutes"));
}

#[test]
fn finds_database_connection() {
    let r = query(&graph(), "database connection", 10);
    // "connection" matches the db file and the connect symbol; "database"
    // matches nothing directly, but the query still resolves via connection.
    assert!(r.files.iter().any(|m| m.name == "src/db/connection.ts"));
    assert!(r.symbols.iter().any(|s| s.name == "connect"));
}

#[test]
fn finds_dependency_and_command() {
    let deps = query(&graph(), "express", 10);
    assert!(deps.dependencies.iter().any(|m| m.name == "express"));
    let cmds = query(&graph(), "dev", 10);
    assert!(cmds.commands.iter().any(|m| m.name == "dev"));
}

#[test]
fn no_match_is_empty() {
    let r = query(&graph(), "kubernetes helm chart", 10);
    assert!(r.is_empty());
}

#[test]
fn respects_limit() {
    let r = query(&graph(), "src", 1);
    assert!(r.areas.len() <= 1);
}
