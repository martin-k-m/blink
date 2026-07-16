mod cli;
mod commands;
mod ui;

use clap::Parser;
use colored::Colorize;

use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Init(args) => commands::init::run(args),
        Command::Scan(args) => commands::scan::run(args),
        Command::Analyze(args) => commands::analyze::run(args),
        Command::Run(args) => commands::run::run(args),
        Command::Build(args) => commands::build::run(args),
    };

    if let Err(err) = result {
        eprintln!("{} {err:#}", "error:".red().bold());
        std::process::exit(1);
    }
}
