use blink_core::{Framework, Project};

/// A short human-readable descriptor for a project's stack, e.g.
/// `React + TypeScript`, `Cargo (Rust)`, or `Python`.
pub fn project_type_label(project: &Project) -> String {
    match project.framework {
        Framework::None => project.language.to_string(),
        Framework::Cargo => format!("Cargo ({})", project.language),
        framework => format!("{framework} + {}", project.language),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use blink_core::{Framework, Language, PackageManager, Project};

    use super::project_type_label;

    fn project(framework: Framework, language: Language) -> Project {
        Project {
            name: "sample".to_string(),
            root: PathBuf::from("."),
            language,
            framework,
            package_manager: PackageManager::Npm,
            dependencies: Vec::new(),
            file_count: 0,
            config_file: "package.json".to_string(),
            is_workspace: false,
            has_virtualenv: false,
        }
    }

    #[test]
    fn combines_framework_and_language() {
        let label = project_type_label(&project(Framework::React, Language::TypeScript));
        assert_eq!(label, "React + TypeScript");
    }

    #[test]
    fn cargo_projects_show_the_language_in_parentheses() {
        let label = project_type_label(&project(Framework::Cargo, Language::Rust));
        assert_eq!(label, "Cargo (Rust)");
    }

    #[test]
    fn falls_back_to_language_alone() {
        let label = project_type_label(&project(Framework::None, Language::Python));
        assert_eq!(label, "Python");
    }

    #[test]
    fn vite_is_treated_like_any_other_framework() {
        let label = project_type_label(&project(Framework::Vite, Language::JavaScript));
        assert_eq!(label, "Vite + JavaScript");
    }
}
