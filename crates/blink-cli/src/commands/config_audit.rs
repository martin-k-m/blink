use anyhow::Result;
use blink_core::BlinkConfig;
use blink_workflow::config_audit;
use colored::Colorize;

use crate::cli::{ConfigAction, ConfigArgs, FormatArgs};
use crate::ui;

pub fn run(args: FormatArgs) -> Result<()> {
    let items = config_audit::audit(&args.path);

    if args.json {
        let json: Vec<_> = items
            .iter()
            .map(|i| {
                serde_json::json!({
                    "name": i.name,
                    "present": i.present,
                    "required": i.required,
                    "note": i.note,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    ui::banner("Blink Configuration Audit");
    println!();
    for item in &items {
        if item.present {
            println!("  {} {}", "\u{2713}".green().bold(), item.name);
        } else if item.required {
            println!(
                "  {} {}  {}",
                "\u{26a0}".yellow().bold(),
                item.name.bold(),
                format!("missing \u{2014} {}", item.note).dimmed()
            );
        } else {
            println!(
                "  {} {}  {}",
                "\u{2192}".dimmed(),
                item.name,
                format!("missing \u{2014} {}", item.note).dimmed()
            );
        }
    }
    println!();
    Ok(())
}

/// `blink config check` — validate the project's blink.toml/.bnk.
pub fn config(args: ConfigArgs) -> Result<()> {
    match args.action {
        ConfigAction::Check { path } => {
            ui::banner("Blink Configuration");
            println!();

            let Some(config_path) = BlinkConfig::config_path(&path) else {
                ui::field("Config", "no blink.toml or .bnk found");
                println!();
                ui::suggestion("Run `blink init` to create one.");
                println!();
                return Ok(());
            };

            let name = config_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            match BlinkConfig::load(&path) {
                Ok(cfg) => {
                    ui::step(format!("{name} is valid"));
                    ui::field("Project", cfg.project.name);
                    ui::field("Commands", ui::format_count(cfg.commands.len()));
                    ui::field("Profiles", ui::format_count(cfg.profiles.len()));
                    ui::field(
                        "Index",
                        if cfg.index.enabled {
                            "enabled"
                        } else {
                            "disabled"
                        },
                    );
                    let context = if !cfg.context.enabled {
                        "disabled".to_string()
                    } else if cfg.context.include.is_empty() {
                        "enabled (whole project)".to_string()
                    } else {
                        format!("enabled (include: {})", cfg.context.include.join(", "))
                    };
                    ui::field("Context", context);

                    // Warn about profiles that reference no commands.
                    let empty: Vec<&String> = cfg
                        .profiles
                        .iter()
                        .filter(|(_, p)| p.commands.is_empty())
                        .map(|(n, _)| n)
                        .collect();
                    if !empty.is_empty() {
                        println!();
                        for name in empty {
                            ui::warning(format!("profile `{name}` has no commands"));
                        }
                    }
                    println!();
                }
                Err(err) => {
                    println!("  {} {name} is invalid", "\u{274c}".red().bold());
                    ui::field("Error", err);
                    println!();
                    anyhow::bail!("invalid configuration");
                }
            }
            Ok(())
        }
    }
}
