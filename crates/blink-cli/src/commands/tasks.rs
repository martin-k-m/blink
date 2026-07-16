use anyhow::{Context, Result};
use blink_core::BlinkConfig;
use blink_workflow::tasks;
use colored::Colorize;

use crate::cli::{FormatArgs, ProfileArgs, TaskArgs};
use crate::proc;
use crate::ui;

/// `blink tasks` — list discovered tasks.
pub fn list(args: FormatArgs) -> Result<()> {
    let discovered = tasks::discover(&args.path);

    if args.json {
        let json: Vec<_> = discovered
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "command": t.command,
                    "source": t.source.to_string(),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    ui::banner("Blink Tasks");
    if discovered.is_empty() {
        println!("\n  No tasks found.\n");
        ui::suggestion("Define commands under `[commands]` in blink.toml, or add package.json scripts / a Makefile.");
        println!();
        return Ok(());
    }
    println!();
    for task in &discovered {
        println!(
            "  {}  {}",
            format!("{:<16}", task.name).bold(),
            task.command.dimmed()
        );
        println!("  {:<16}  {}", "", format!("via {}", task.source).dimmed());
    }
    println!();
    ui::footer("Run", "blink task <name>");
    Ok(())
}

/// `blink task <name>` — run a discovered task.
pub fn run(args: TaskArgs) -> Result<()> {
    let Some(task) = tasks::find(&args.path, &args.name) else {
        anyhow::bail!(
            "no task named `{}` (run `blink tasks` to see what's available)",
            args.name
        );
    };

    ui::banner(&format!("Blink Task \u{2014} {}", task.name));
    ui::field("Command", &task.command);
    ui::field("Source", task.source.to_string());
    println!();

    if args.dry_run {
        ui::step("dry run \u{2014} not executed");
        println!();
        return Ok(());
    }

    let status = proc::run_shell(&task.command, &args.path)
        .with_context(|| format!("failed to launch `{}`", task.command))?;
    exit_like(status)
}

/// `blink profile <name>` — run a `[profiles]` command sequence.
pub fn profile(args: ProfileArgs) -> Result<()> {
    let config = BlinkConfig::load(&args.path)
        .context("no blink.toml/.bnk found (profiles are defined there)")?;
    let Some(profile) = config.profiles.get(&args.name) else {
        anyhow::bail!("no profile named `{}` in configuration", args.name);
    };
    if profile.commands.is_empty() {
        anyhow::bail!("profile `{}` defines no commands", args.name);
    }

    ui::banner(&format!("Blink Profile \u{2014} {}", args.name));
    for (i, command) in profile.commands.iter().enumerate() {
        ui::field(&format!("Step {}", i + 1), command);
    }
    println!();

    if args.dry_run {
        ui::step("dry run \u{2014} not executed");
        println!();
        return Ok(());
    }

    for command in &profile.commands {
        println!("  {} {command}", "\u{25b6}".truecolor(255, 138, 0));
        let status = proc::run_shell(command, &args.path)
            .with_context(|| format!("failed to launch `{command}`"))?;
        if !status.success() {
            let code = status.code().unwrap_or(1);
            anyhow::bail!("`{command}` exited with status {code}; stopping profile");
        }
    }
    ui::footer("Result", "profile complete");
    Ok(())
}

/// Exit the process mirroring a child's exit code, so `blink task test` is
/// usable in scripts and CI just like the underlying command.
fn exit_like(status: std::process::ExitStatus) -> Result<()> {
    match status.code() {
        Some(0) => Ok(()),
        Some(code) => std::process::exit(code),
        None => std::process::exit(1),
    }
}
