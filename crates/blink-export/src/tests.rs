use std::path::PathBuf;

use blink_context::{
    Area, ConfigInfo, ContextGraph, DependencyNode, FileNode, ProjectInfo, Reference, Stats,
};

use super::*;

fn graph() -> ContextGraph {
    ContextGraph {
        project: ProjectInfo {
            name: "demo".into(),
            language: "TypeScript".into(),
            framework: Some("React".into()),
            package_manager: "npm".into(),
            is_workspace: false,
        },
        stats: Stats {
            files: 2,
            source_files: 2,
            lines: 30,
            symbols: 2,
            size_bytes: 2048,
        },
        config: ConfigInfo {
            present: true,
            file: Some(".bnk".into()),
            context_enabled: true,
            include: Vec::new(),
        },
        areas: vec![
            Area {
                path: "src".into(),
                files: 1,
                lines: 10,
                symbols: 1,
                size_bytes: 1024,
                languages: vec!["TypeScript".into()],
            },
            Area {
                path: "src/api".into(),
                files: 1,
                lines: 20,
                symbols: 1,
                size_bytes: 1024,
                languages: vec!["TypeScript".into()],
            },
        ],
        dependencies: vec![DependencyNode {
            name: "react".into(),
            version: "^18.0.0".into(),
            dev: false,
        }],
        commands: Vec::new(),
        files: vec![
            FileNode {
                path: "src/app.ts".into(),
                area: "src".into(),
                lang: Some("TypeScript".into()),
                lines: 10,
                size_bytes: 1024,
                symbols: Vec::new(),
            },
            FileNode {
                path: "src/api/routes.ts".into(),
                area: "src/api".into(),
                lang: Some("TypeScript".into()),
                lines: 20,
                size_bytes: 1024,
                symbols: Vec::new(),
            },
        ],
        references: vec![Reference {
            from: "src/app.ts".into(),
            to: "src/api/routes.ts".into(),
        }],
        root: PathBuf::new(),
    }
}

#[test]
fn json_round_trips() {
    let out = export(&graph(), ExportFormat::Json);
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(value["project"]["name"], "demo");
}

#[test]
fn yaml_is_valid_block_style() {
    let out = export(&graph(), ExportFormat::Yaml);
    assert!(out.contains("\"name\": \"demo\""), "yaml:\n{out}");
    // No tab characters, values quoted — a safe block document.
    assert!(!out.contains('\t'));
}

#[test]
fn markdown_has_sections() {
    let out = export(&graph(), ExportFormat::Markdown);
    assert!(out.starts_with("# Blink Context — demo"));
    assert!(out.contains("## Areas"));
    assert!(out.contains("`react`"));
}

#[test]
fn mermaid_has_edge_between_areas() {
    let out = export(&graph(), ExportFormat::Graph);
    assert!(out.starts_with("graph TD"));
    // One reference crosses from `src` into `src/api`.
    assert!(out.contains("-->|1|"), "mermaid:\n{out}");
}

#[test]
fn format_parsing_and_filenames() {
    assert_eq!(
        "md".parse::<ExportFormat>().unwrap(),
        ExportFormat::Markdown
    );
    assert_eq!("yml".parse::<ExportFormat>().unwrap(), ExportFormat::Yaml);
    assert_eq!(
        "mermaid".parse::<ExportFormat>().unwrap(),
        ExportFormat::Graph
    );
    assert!("toml".parse::<ExportFormat>().is_err());
    assert_eq!(ExportFormat::Json.default_filename(), "blink-context.json");
}
