//! Project configuration audit: which of the files that make a repository
//! approachable and CI-ready are present. Purely presence checks — Blink does
//! not judge the *contents* of these files.

use std::path::Path;

/// One audited configuration file (or family of files).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditItem {
    pub name: String,
    pub present: bool,
    /// Whether its absence is a real gap (`README`) vs. a suggestion
    /// (`.editorconfig`).
    pub required: bool,
    pub note: String,
}

/// Audit the standard project-config files in `root`.
pub fn audit(root: &Path) -> Vec<AuditItem> {
    let mut items = Vec::new();

    let mut check = |name: &str, required: bool, present: bool, note: &str| {
        items.push(AuditItem {
            name: name.to_string(),
            present,
            required,
            note: note.to_string(),
        });
    };

    check(
        "README",
        true,
        any_exists(root, &["README.md", "README", "README.rst", "README.txt"]),
        "explains what the project is and how to use it",
    );
    check(
        "LICENSE",
        false,
        any_exists(root, &["LICENSE", "LICENSE.md", "LICENSE.txt", "COPYING"]),
        "clarifies how others may use the code",
    );
    check(
        "CONTRIBUTING",
        false,
        any_exists(
            root,
            &["CONTRIBUTING.md", "CONTRIBUTING", "docs/CONTRIBUTING.md"],
        ),
        "tells contributors how to get started",
    );
    check(
        ".gitignore",
        true,
        root.join(".gitignore").is_file(),
        "keeps build output and secrets out of version control",
    );
    check(
        ".editorconfig",
        false,
        root.join(".editorconfig").is_file(),
        "keeps formatting consistent across editors",
    );
    check(
        "CI configuration",
        false,
        has_ci_config(root),
        "runs checks automatically on every change",
    );

    items
}

fn any_exists(root: &Path, names: &[&str]) -> bool {
    names.iter().any(|n| root.join(n).is_file())
}

fn has_ci_config(root: &Path) -> bool {
    if any_exists(
        root,
        &[
            ".gitlab-ci.yml",
            "azure-pipelines.yml",
            ".circleci/config.yml",
            "Jenkinsfile",
        ],
    ) {
        return true;
    }
    // GitHub Actions: any workflow file under .github/workflows.
    let workflows = root.join(".github").join("workflows");
    if let Ok(entries) = std::fs::read_dir(&workflows) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension() {
                if ext == "yml" || ext == "yaml" {
                    return true;
                }
            }
        }
    }
    false
}
