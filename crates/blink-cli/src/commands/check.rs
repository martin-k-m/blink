use anyhow::{Context, Result};
use blink_core::{Language, Project, ProjectDetector};
use blink_workflow::{on_path, tasks};
use colored::Colorize;

use crate::cli::CheckArgs;
use crate::proc;
use crate::ui;

/// A validation step: a human label and the shell command that implements it.
struct Step {
    label: &'static str,
    command: String,
}

pub fn run(args: CheckArgs) -> Result<()> {
    let project = ProjectDetector::new()
        .detect(&args.path)
        .with_context(|| format!("could not inspect {}", args.path.display()))?;

    let steps = plan(&project, &args.path);

    ui::banner("Blink Check");
    if steps.is_empty() {
        println!("\n  No checks available for this project.\n");
        ui::suggestion(
            "Add package.json scripts (lint/test) or a Makefile, or use a supported toolchain.",
        );
        println!();
        return Ok(());
    }

    println!();
    let mut failures = 0;
    for step in &steps {
        println!(
            "  {} {}  {}",
            "\u{25b6}".truecolor(255, 45, 141),
            step.label.bold(),
            step.command.dimmed()
        );
        match proc::run_shell(&step.command, &args.path) {
            Ok(status) if status.success() => ui::step(format!("{} passed", step.label)),
            Ok(status) => {
                failures += 1;
                ui::warning(format!(
                    "{} failed (exit {})",
                    step.label,
                    status.code().unwrap_or(-1)
                ));
            }
            Err(err) => {
                failures += 1;
                ui::warning(format!("{} could not run: {err}", step.label));
            }
        }
        println!();
    }

    if failures == 0 {
        ui::footer("Result", format!("{} check(s) passed", steps.len()));
        Ok(())
    } else {
        ui::footer(
            "Result",
            format!("{failures} of {} check(s) failed", steps.len()),
        );
        std::process::exit(1);
    }
}

/// Decide which checks to run: prefer the project's own declared tasks
/// (package.json scripts, etc.), falling back to conventional toolchain
/// commands. Only includes steps whose tool is actually available.
fn plan(project: &Project, root: &std::path::Path) -> Vec<Step> {
    let mut steps = Vec::new();
    let discovered = tasks::discover(root);
    let task_cmd = |name: &str| {
        discovered
            .iter()
            .find(|t| t.name == name)
            .map(|t| t.command.clone())
    };

    match project.language {
        Language::Rust if on_path("cargo") => {
            steps.push(Step {
                label: "format",
                command: "cargo fmt --all -- --check".to_string(),
            });
            steps.push(Step {
                label: "lint",
                command: "cargo clippy --all-targets -- -D warnings".to_string(),
            });
            steps.push(Step {
                label: "tests",
                command: "cargo test".to_string(),
            });
        }
        Language::TypeScript | Language::JavaScript => {
            if let Some(cmd) = task_cmd("lint") {
                steps.push(Step {
                    label: "lint",
                    command: cmd,
                });
            }
            if let Some(cmd) = task_cmd("test").or_else(|| task_cmd("tests")) {
                steps.push(Step {
                    label: "tests",
                    command: cmd,
                });
            }
        }
        Language::Python => {
            if on_path("ruff") {
                steps.push(Step {
                    label: "lint",
                    command: "ruff check .".to_string(),
                });
            }
            if on_path("pytest") {
                steps.push(Step {
                    label: "tests",
                    command: "pytest".to_string(),
                });
            }
        }
        _ => {}
    }

    steps
}
