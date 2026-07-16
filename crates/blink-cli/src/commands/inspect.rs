use anyhow::{Context, Result};
use blink_analyzer::format_bytes;
use blink_core::{BlinkConfig, Language, Project, ProjectDetector};
use blink_index::Index;
use blink_workflow::{git, tasks};
use colored::Colorize;

use crate::cli::InspectArgs;
use crate::indexing::ensure_index;
use crate::ui;

pub fn run(args: InspectArgs) -> Result<()> {
    let spinner = ui::spinner("Inspecting project...");
    let project = ProjectDetector::new()
        .detect(&args.path)
        .with_context(|| format!("could not inspect {}", args.path.display()))?;
    let index = ensure_index(&args.path)?;
    spinner.finish_and_clear();

    let run_cmd = run_command(&project, &args.path);
    let entry_points = entry_points(&index, &project);
    let task_list = tasks::discover(&args.path);

    if args.json {
        let json = serde_json::json!({
            "name": project.name,
            "language": project.language.to_string(),
            "framework": project.framework.to_string(),
            "package_manager": project.package_manager.to_string(),
            "is_workspace": project.is_workspace,
            "files": index.file_count(),
            "lines": index.total_lines(),
            "symbols": index.symbol_count(),
            "size_bytes": index.total_size(),
            "dependencies": project.dependency_count(),
            "run_command": run_cmd,
            "entry_points": entry_points,
            "tasks": task_list.iter().map(|t| t.name.clone()).collect::<Vec<_>>(),
            "languages": index.language_breakdown().iter().map(|(lang, (files, lines))| {
                serde_json::json!({ "language": lang.to_string(), "files": files, "lines": lines })
            }).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    ui::banner(&format!("Blink Inspect \u{2014} {}", project.name));

    // What is this project?
    let mut descriptor = project.language.to_string();
    if project.framework.to_string() != "None" {
        descriptor = format!("{descriptor} / {}", project.framework);
    }
    if project.is_workspace {
        descriptor.push_str(" workspace");
    }
    ui::field("Project", descriptor);
    ui::field("Package manager", project.package_manager.to_string());
    if let Ok(config) = BlinkConfig::load(&args.path) {
        if let Some(t) = config.project.r#type {
            ui::field("Type", t);
        }
    }

    // Statistics (measured from the index).
    ui::field("Files", ui::format_count(index.file_count()));
    ui::field("Lines of code", ui::format_count(index.total_lines()));
    ui::field("Symbols", ui::format_count(index.symbol_count()));
    ui::field("Size", format_bytes(index.total_size()));
    ui::field("Dependencies", ui::format_count(project.dependency_count()));

    // How do I run it?
    println!();
    println!("  {}", "How to run".bold());
    if let Some(cmd) = &run_cmd {
        ui::suggestion(cmd);
    } else {
        ui::field("", "no obvious run command detected");
    }

    // Where do I start?
    if !entry_points.is_empty() {
        println!();
        println!("  {}", "Entry points".bold());
        for ep in &entry_points {
            println!("    {ep}");
        }
    }

    // Available tasks.
    if !task_list.is_empty() {
        println!();
        println!("  {}", "Tasks".bold());
        for task in task_list.iter().take(10) {
            println!(
                "    {}  {}",
                task.name.bold(),
                format!("({})", task.source).dimmed()
            );
        }
        if task_list.len() > 10 {
            ui::field(
                "",
                format!("... and {} more (blink tasks)", task_list.len() - 10),
            );
        }
    }

    // Git snapshot.
    if git::available(&args.path) {
        if let Some(count) = git::commit_count(&args.path) {
            println!();
            ui::field("Git commits", ui::format_count(count));
        }
    }

    println!();
    ui::footer("Next", "blink optimize  ·  blink health  ·  blink doctor");
    Ok(())
}

/// The natural "start" command for the project's ecosystem, if determinable.
fn run_command(project: &Project, root: &std::path::Path) -> Option<String> {
    // A package.json "dev"/"start" script wins when present.
    if matches!(
        project.language,
        Language::TypeScript | Language::JavaScript
    ) {
        let tasks = tasks::discover(root);
        for name in ["dev", "start"] {
            if let Some(task) = tasks.iter().find(|t| t.name == name) {
                return Some(task.command.clone());
            }
        }
    }
    match project.language {
        Language::Rust => Some("cargo run".to_string()),
        Language::TypeScript | Language::JavaScript => {
            Some(format!("{} run dev", project.package_manager))
        }
        Language::Python => {
            for candidate in ["main.py", "app.py", "manage.py"] {
                if root.join(candidate).is_file() {
                    return Some(format!("python {candidate}"));
                }
            }
            None
        }
        Language::Unknown => None,
    }
}

/// Conventional entry-point files that actually exist in the index.
fn entry_points(index: &Index, project: &Project) -> Vec<String> {
    let candidates: &[&str] = match project.language {
        Language::Rust => &["src/main.rs", "src/lib.rs"],
        Language::TypeScript | Language::JavaScript => &[
            "src/index.ts",
            "src/index.js",
            "src/main.ts",
            "src/main.tsx",
            "src/App.tsx",
            "index.js",
            "index.ts",
        ],
        Language::Python => &["main.py", "app.py", "__main__.py", "manage.py"],
        Language::Unknown => &[],
    };
    candidates
        .iter()
        .filter(|c| index.files.contains_key(**c))
        .map(|c| c.to_string())
        .collect()
}
