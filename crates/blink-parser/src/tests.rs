use crate::{
    parse_cargo_lock, parse_cargo_manifest, parse_npm_lock, parse_package_json,
    parse_requirements_txt,
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
