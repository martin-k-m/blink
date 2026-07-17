use anyhow::{Context, Result};
use blink_core::{Language, PackageManager, Project, ProjectDetector};
use blink_workflow::{env, on_path};
use colored::Colorize;

use crate::cli::SetupArgs;
use crate::proc;
use crate::ui;

pub fn run(args: SetupArgs) -> Result<()> {
    let project = ProjectDetector::new()
        .detect(&args.path)
        .with_context(|| format!("could not inspect {}", args.path.display()))?;

    ui::banner("Blink Setup");
    println!();

    // Copy .env.example -> .env if the latter is missing, so a fresh clone has
    // a starting point (values still need filling in by the developer).
    let env_report = env::compare(&args.path);
    let copy_env = env_report.has_example && !env_report.has_env;

    let install = install_command(&project);

    // Show the plan first — Blink never runs hidden steps.
    if let Some(cmd) = &install {
        ui::field("Install", cmd);
    }
    if copy_env {
        ui::field("Env", "copy .env.example -> .env");
    }
    if install.is_none() && !copy_env {
        println!("  {} Nothing to set up\n", "\u{2713}".green().bold());
        return Ok(());
    }

    println!();
    if !args.yes && !proc::confirm("Proceed with setup?") {
        ui::step("cancelled");
        println!();
        return Ok(());
    }

    if copy_env {
        std::fs::copy(args.path.join(".env.example"), args.path.join(".env"))
            .context("could not copy .env.example to .env")?;
        ui::step("created .env from .env.example");
    }

    if let Some(cmd) = &install {
        println!("  {} {cmd}", "\u{25b6}".truecolor(255, 45, 141));
        let status = proc::run_shell(cmd, &args.path)
            .with_context(|| format!("failed to launch `{cmd}`"))?;
        if !status.success() {
            anyhow::bail!("`{cmd}` exited with status {}", status.code().unwrap_or(-1));
        }
        ui::step("dependencies installed");
    }

    println!();
    ui::footer("Result", "setup complete");
    Ok(())
}

/// The dependency-install command for the project's package manager, if its
/// tool is available on PATH.
fn install_command(project: &Project) -> Option<String> {
    let (tool, cmd) = match project.package_manager {
        PackageManager::Npm => ("npm", "npm install"),
        PackageManager::Pnpm => ("pnpm", "pnpm install"),
        PackageManager::Yarn => ("yarn", "yarn install"),
        PackageManager::Cargo => ("cargo", "cargo fetch"),
        PackageManager::Pip => ("pip", "pip install -r requirements.txt"),
        PackageManager::Unknown => return None,
    };
    // For Python, only offer the pip install if requirements.txt is the manifest.
    if project.language == Language::Python && project.config_file != "requirements.txt" {
        return None;
    }
    on_path(tool).then(|| cmd.to_string())
}
