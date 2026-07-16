//! Environment diagnostics: can this project actually be developed here? Checks
//! that the runtimes, package managers, and tools the project needs are on
//! `PATH`, plus any environment-variable *names* (never values) declared in
//! `.env.example` but not configured.

use std::path::Path;

use blink_core::{Language, PackageManager, Project};

use crate::env;
use crate::fs_util::on_path;

/// Outcome of a single environment check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    /// The tool/variable is present.
    Ok,
    /// Absent. `required` distinguishes a blocker from a nice-to-have.
    Missing,
}

/// One diagnostic line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    /// What was checked, e.g. `cargo`, `Docker`, or `env: DATABASE_URL`.
    pub name: String,
    pub status: CheckStatus,
    /// Whether a `Missing` result should be treated as a hard problem.
    pub required: bool,
    /// Short human explanation of why this was checked.
    pub detail: String,
}

impl Check {
    fn tool(name: &str, required: bool, detail: &str) -> Self {
        Check {
            status: if on_path(name) {
                CheckStatus::Ok
            } else {
                CheckStatus::Missing
            },
            name: name.to_string(),
            required,
            detail: detail.to_string(),
        }
    }
}

/// Full doctor report for a project.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DoctorReport {
    pub checks: Vec<Check>,
}

impl DoctorReport {
    /// True when no *required* check is missing — i.e. development can proceed.
    pub fn is_healthy(&self) -> bool {
        !self
            .checks
            .iter()
            .any(|c| c.required && c.status == CheckStatus::Missing)
    }

    pub fn missing_required(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| c.required && c.status == CheckStatus::Missing)
            .count()
    }

    pub fn missing_optional(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| !c.required && c.status == CheckStatus::Missing)
            .count()
    }
}

/// Run environment checks for `project` rooted at `root`.
pub fn diagnose(project: &Project, root: &Path) -> DoctorReport {
    let mut checks = Vec::new();

    // Git is relevant to essentially every project, but not strictly required
    // to build, so it's a warning rather than a blocker.
    checks.push(Check::tool("git", false, "version control"));

    // Language runtime + compiler.
    match project.language {
        Language::Rust => {
            checks.push(Check::tool("cargo", true, "Rust build tool"));
            checks.push(Check::tool("rustc", true, "Rust compiler"));
        }
        Language::TypeScript | Language::JavaScript => {
            checks.push(Check::tool("node", true, "JavaScript runtime"));
        }
        Language::Python => {
            let py = if on_path("python") {
                "python"
            } else {
                "python3"
            };
            checks.push(Check::tool(py, true, "Python interpreter"));
        }
        Language::Unknown => {}
    }

    // The project's specific package manager.
    match project.package_manager {
        PackageManager::Npm => checks.push(Check::tool("npm", true, "package manager")),
        PackageManager::Pnpm => checks.push(Check::tool("pnpm", true, "package manager")),
        PackageManager::Yarn => checks.push(Check::tool("yarn", true, "package manager")),
        PackageManager::Pip => checks.push(Check::tool("pip", false, "package manager")),
        // cargo already covered above; nothing extra for unknown.
        PackageManager::Cargo | PackageManager::Unknown => {}
    }

    // Docker only matters if the project ships container config.
    if root.join("Dockerfile").is_file()
        || root.join("docker-compose.yml").is_file()
        || root.join("docker-compose.yaml").is_file()
        || root.join("compose.yaml").is_file()
    {
        checks.push(Check::tool(
            "docker",
            false,
            "container tooling (compose file present)",
        ));
    }

    // Environment variables — names only, never values.
    let env_report = env::compare(root);
    for name in env_report.missing {
        checks.push(Check {
            name: format!("env: {name}"),
            status: CheckStatus::Missing,
            required: false,
            detail: "declared in .env.example, absent from .env".to_string(),
        });
    }

    DoctorReport { checks }
}
