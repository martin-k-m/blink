use crate::{
    parse_cargo_lock, parse_cargo_manifest, parse_npm_lock, parse_npm_lock_direct,
    parse_package_json, parse_pnpm_lock, parse_requirements_txt, parse_yarn_lock,
};

#[test]
fn parses_cargo_manifest_dependencies() {
    let manifest = parse_cargo_manifest(
        r#"
            [package]
            name = "sample"

            [dependencies]
            serde = "1"
            tokio = { version = "1", features = ["full"] }

            [dev-dependencies]
            tempfile = "3"
        "#,
    );

    assert_eq!(manifest.name.as_deref(), Some("sample"));
    assert_eq!(manifest.dependencies.len(), 3);
    let tokio = manifest
        .dependencies
        .iter()
        .find(|d| d.name == "tokio")
        .unwrap();
    assert_eq!(tokio.version, "1");
    assert!(!tokio.dev);
    let tempfile = manifest
        .dependencies
        .iter()
        .find(|d| d.name == "tempfile")
        .unwrap();
    assert!(tempfile.dev);
}

#[test]
fn detects_cargo_workspace_table() {
    let manifest = parse_cargo_manifest(
        r#"
            [workspace]
            members = ["crates/*"]
        "#,
    );
    assert!(manifest.is_workspace);

    let manifest = parse_cargo_manifest(
        r#"
            [package]
            name = "sample"
        "#,
    );
    assert!(!manifest.is_workspace);
}

#[test]
fn malformed_cargo_manifest_falls_back_to_empty() {
    let manifest = parse_cargo_manifest("not valid toml {{{");

    assert!(manifest.name.is_none());
    assert!(manifest.dependencies.is_empty());
}

#[test]
fn parses_package_json_dependencies() {
    let manifest = parse_package_json(
        r#"{
            "name": "sample-app",
            "dependencies": { "react": "^18.0.0" },
            "devDependencies": { "typescript": "^5.0.0" }
        }"#,
    )
    .unwrap();

    assert_eq!(manifest.name.as_deref(), Some("sample-app"));
    assert!(manifest.has_dependency("react"));
    assert!(manifest.has_dependency("typescript"));
    assert!(!manifest.has_dependency("vue"));
}

#[test]
fn malformed_package_json_is_an_error() {
    let result = parse_package_json("not json at all");

    assert!(result.is_err());
}

#[test]
fn parses_requirements_txt() {
    let deps =
        parse_requirements_txt("# comment\nflask==2.3.0\nrequests>=2.28\nnumpy\n\npytest~=7.4");

    assert_eq!(deps.len(), 4);
    assert_eq!(deps[0].name, "flask");
    assert_eq!(deps[0].version, "2.3.0");
    assert_eq!(deps[2].name, "numpy");
    assert_eq!(deps[2].version, "*");
}

#[test]
fn parses_cargo_lock_entries() {
    let entries = parse_cargo_lock(
        r#"
            [[package]]
            name = "serde"
            version = "1.0.190"

            [[package]]
            name = "syn"
            version = "2.0.0"
        "#,
    );

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].name, "serde");
    assert_eq!(entries[1].version, "2.0.0");
}

#[test]
fn parses_npm_lock_entries() {
    let entries = parse_npm_lock(
        r#"{
            "packages": {
                "": { "name": "root" },
                "node_modules/react": { "version": "18.2.0" },
                "node_modules/react-dom": { "version": "18.2.0" }
            }
        }"#,
    );

    assert_eq!(entries.len(), 2);
    assert!(entries
        .iter()
        .any(|e| e.name == "react" && e.version == "18.2.0"));
}

#[test]
fn cargo_lock_records_edges_and_workspace_members() {
    let entries = parse_cargo_lock(
        r#"
            [[package]]
            name = "my-app"
            version = "0.1.0"
            dependencies = ["ratatui"]

            [[package]]
            name = "ratatui"
            version = "0.29.0"
            source = "registry+https://github.com/rust-lang/crates.io-index"
            dependencies = ["lru 0.12.5"]

            [[package]]
            name = "lru"
            version = "0.12.5"
            source = "registry+https://github.com/rust-lang/crates.io-index"
        "#,
    );

    assert_eq!(entries.len(), 3);
    // A `[[package]]` without a `source` is a workspace member, not a
    // downloaded package.
    assert!(entries[0].local);
    assert!(!entries[1].local);
    assert_eq!(entries[0].dependencies, vec!["ratatui".to_string()]);
    // `"lru 0.12.5"` keeps only the name.
    assert_eq!(entries[1].dependencies, vec!["lru".to_string()]);
    assert!(entries[2].dependencies.is_empty());
}

#[test]
fn npm_lock_records_edges_and_direct_dependencies() {
    let raw = r#"{
        "packages": {
            "": {
                "name": "root",
                "dependencies": { "react": "^18.2.0" },
                "devDependencies": { "vite": "^5.2.0" }
            },
            "node_modules/react": {
                "version": "18.2.0",
                "dependencies": { "loose-envify": "^1.1.0" }
            },
            "node_modules/loose-envify": { "version": "1.4.0" },
            "node_modules/vite": { "version": "5.2.0" }
        }
    }"#;

    let entries = parse_npm_lock(raw);
    assert_eq!(entries.len(), 3);
    let react = entries.iter().find(|e| e.name == "react").unwrap();
    assert_eq!(react.dependencies, vec!["loose-envify".to_string()]);

    let direct = parse_npm_lock_direct(raw);
    assert_eq!(direct, vec!["react".to_string(), "vite".to_string()]);
}

#[test]
fn parses_yarn_v1_lock_entries() {
    let entries = parse_yarn_lock(
        r#"
# THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.
# yarn lockfile v1


"@babel/code-frame@^7.0.0", "@babel/code-frame@^7.10.4":
  version "7.24.7"
  resolved "https://registry.yarnpkg.com/@babel/code-frame/-/code-frame-7.24.7.tgz"

react@^18.2.0:
  version "18.2.0"
  resolved "https://registry.yarnpkg.com/react/-/react-18.2.0.tgz"
  dependencies:
    loose-envify "^1.1.0"
"#,
    );

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].name, "@babel/code-frame");
    assert_eq!(entries[0].version, "7.24.7");
    assert_eq!(entries[1].name, "react");
    assert_eq!(entries[1].version, "18.2.0");
    // A nested `dependencies:` block is metadata, not a resolved entry.
    assert!(!entries.iter().any(|e| e.name == "loose-envify"));
}

#[test]
fn parses_yarn_berry_lock_entries() {
    let entries = parse_yarn_lock(
        r#"
__metadata:
  version: 8

"react@npm:^18.2.0":
  version: 18.2.0
  resolution: "react@npm:18.2.0"
"#,
    );

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "react");
    assert_eq!(entries[0].version, "18.2.0");
}

#[test]
fn parses_pnpm_lock_entries_across_formats() {
    // v6/v9 style keys, plus a peer-dependency suffix that must be stripped.
    let entries = parse_pnpm_lock(
        r#"
lockfileVersion: '9.0'

importers:
  .:
    dependencies:
      react:
        specifier: ^18.2.0
        version: 18.2.0

packages:

  react@18.2.0:
    resolution: {integrity: sha512-abc}

  react-dom@18.2.0(react@18.2.0):
    resolution: {integrity: sha512-def}

  '@types/react@18.2.0':
    resolution: {integrity: sha512-ghi}

snapshots:

  react@18.2.0: {}

  react-dom@18.2.0(react@18.2.0):
    dependencies:
      react: 18.2.0
"#,
    );

    // The `snapshots:` block repeats the same packages; an inline `: {}`
    // value must not end up glued onto the version.
    assert_eq!(entries.len(), 3);
    assert!(entries
        .iter()
        .any(|e| e.name == "react" && e.version == "18.2.0"));
    assert!(entries
        .iter()
        .any(|e| e.name == "react-dom" && e.version == "18.2.0"));
    assert!(entries
        .iter()
        .any(|e| e.name == "@types/react" && e.version == "18.2.0"));
    // The `importers:` block must not be mistaken for resolved packages.
    assert!(!entries.iter().any(|e| e.name == "."));
}

#[test]
fn parses_pnpm_v5_slash_separated_keys() {
    let entries = parse_pnpm_lock(
        r#"
lockfileVersion: 5.4

packages:

  /react/18.2.0:
    resolution: {integrity: sha512-abc}

  /@types/react/18.2.0:
    resolution: {integrity: sha512-def}
"#,
    );

    assert_eq!(entries.len(), 2);
    assert!(entries
        .iter()
        .any(|e| e.name == "react" && e.version == "18.2.0"));
    assert!(entries
        .iter()
        .any(|e| e.name == "@types/react" && e.version == "18.2.0"));
}
