mod analysis;
mod cli;
mod commands;
mod ui;

use clap::{CommandFactory, Parser};
use colored::Colorize;

use cli::{Cli, Command};

fn main() {
    try_run_as_plugin();

    let cli = Cli::parse();

    let result = match cli.command {
        Command::Init(args) => commands::init::run(args),
        Command::Scan(args) => commands::scan::run(args),
        Command::Analyze(args) => commands::analyze::run(args),
        Command::Run(args) => commands::run::run(args),
        Command::Build(args) => commands::build::run(args),
        Command::Deps(args) => commands::deps::run(args),
        Command::Health(args) => commands::health::run(args),
        Command::Recommend(args) => commands::recommend::run(args),
        Command::Watch(args) => commands::watch::run(args),
        Command::Ci(args) => commands::ci::run(args),
        Command::Security(args) => commands::security::run(args),
        Command::Report(args) => commands::report::run(args),
        Command::Plugins(args) => commands::plugins::run(args),
        Command::Benchmark(args) => commands::benchmark::run(args),
        Command::Dashboard(args) => commands::dashboard::run(args),
    };

    if let Err(err) = result {
        eprintln!("{} {err:#}", "error:".red().bold());
        std::process::exit(1);
    }
}

/// If the first argument isn't a recognized built-in subcommand (or a
/// `-`-prefixed flag like `--help`), check whether it matches an installed
/// plugin (`blink-<name>` on PATH or in `~/.blink/plugins`) and, if so, run
/// it and exit — never returning to fall through to `Cli::parse()`, which
/// would otherwise reject it as an unknown subcommand.
fn try_run_as_plugin() {
    let args: Vec<String> = std::env::args().collect();
    let Some(first) = args.get(1) else {
        return;
    };
    if first.starts_with('-') || is_builtin_command(first) {
        return;
    }
    let Some(plugin) = blink_plugin::find_plugin(first) else {
        return;
    };

    match blink_plugin::run_plugin(&plugin, &args[2..]) {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("{} {err}", "error:".red().bold());
            std::process::exit(1);
        }
    }
}

fn is_builtin_command(name: &str) -> bool {
    Cli::command()
        .get_subcommands()
        .any(|cmd| cmd.get_name() == name || cmd.get_all_aliases().any(|alias| alias == name))
}
