use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "blink",
    version,
    about = "A Rust-powered developer acceleration toolkit.",
    long_about = "Blink removes friction between writing code and running software."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a blink.toml configuration for a project.
    Init(InitArgs),
    /// Detect the project in a directory and report what Blink found.
    Scan(ScanArgs),
    /// Analyze dependency health and produce recommendations.
    Analyze(AnalyzeArgs),
    /// Start Blink's development server with file watching.
    Run(RunArgs),
    /// Run a cache-aware build pass.
    Build(BuildArgs),
}

#[derive(Args)]
pub struct InitArgs {
    /// Directory to initialize (created if it doesn't exist).
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(Args)]
pub struct ScanArgs {
    /// Directory to scan.
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(Args)]
pub struct AnalyzeArgs {
    /// Directory to analyze.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Check crates.io / the npm registry for outdated package versions.
    /// Requires network access.
    #[arg(long)]
    pub online: bool,
}

#[derive(Args)]
pub struct RunArgs {
    /// Project directory to serve.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Port to listen on. Defaults to blink.toml's [server].port, or 3000.
    #[arg(short, long)]
    pub port: Option<u16>,
}

#[derive(Args)]
pub struct BuildArgs {
    /// Project directory to build.
    #[arg(default_value = ".")]
    pub path: PathBuf,
}
