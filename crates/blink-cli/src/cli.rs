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
    /// Show a focused dependency report (counts, largest packages, issues).
    Deps(DepsArgs),
    /// Show a project health score with sub-scores and suggestions.
    Health(HealthArgs),
    /// Show categorized, rule-based recommendations.
    Recommend(RecommendArgs),
    /// Re-run analysis whenever a file changes (no dev server).
    Watch(WatchArgs),
    /// Run analysis for CI: exits 0 (pass), 1 (warnings), or 2 (failure).
    Ci(CiArgs),
    /// Check dependencies against OSV.dev for known vulnerabilities.
    Security(SecurityArgs),
    /// Export a full project report as JSON, Markdown, or HTML.
    Report(ReportArgs),
    /// List or install plugins (executables named `blink-<name>`).
    Plugins(PluginsArgs),
    /// Measure Blink's own startup, scan, and analysis performance.
    Benchmark(BenchmarkArgs),
    /// Open an interactive terminal dashboard for the project.
    Dashboard(DashboardArgs),
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

    /// Print extra diagnostic detail: the resolved absolute path, which
    /// manifest file was matched, and the ignore rules in effect.
    #[arg(short, long)]
    pub verbose: bool,
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

    /// Print the report as JSON instead of formatted terminal output.
    #[arg(long)]
    pub json: bool,
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

#[derive(Args)]
pub struct DepsArgs {
    /// Directory to analyze.
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(Args)]
pub struct HealthArgs {
    /// Directory to analyze.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Print the report as JSON instead of formatted terminal output.
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct RecommendArgs {
    /// Directory to analyze.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Also check for outdated packages and known vulnerabilities.
    /// Requires network access.
    #[arg(long)]
    pub online: bool,
}

#[derive(Args)]
pub struct WatchArgs {
    /// Project directory to watch.
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(Args)]
pub struct CiArgs {
    /// Directory to analyze.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Also check for outdated packages and known vulnerabilities.
    /// Requires network access.
    #[arg(long)]
    pub online: bool,
}

#[derive(Args)]
pub struct SecurityArgs {
    /// Directory to check.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Print the report as JSON instead of formatted terminal output.
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct ReportArgs {
    /// Directory to report on.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Export as JSON.
    #[arg(long, conflicts_with_all = ["markdown", "html"])]
    pub json: bool,

    /// Export as Markdown.
    #[arg(long, conflicts_with_all = ["json", "html"])]
    pub markdown: bool,

    /// Export as a self-contained HTML page.
    #[arg(long, conflicts_with_all = ["json", "markdown"])]
    pub html: bool,

    /// Write the report to this file instead of stdout.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Also check for outdated packages. Requires network access.
    #[arg(long)]
    pub online: bool,
}

#[derive(Args)]
pub struct PluginsArgs {
    #[command(subcommand)]
    pub action: Option<PluginsAction>,
}

#[derive(Args)]
pub struct DashboardArgs {
    /// Project directory to show.
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(Args)]
pub struct BenchmarkArgs {
    /// Directory to benchmark against.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// How many times to launch a fresh `blink` process to measure startup
    /// time. Reported as the minimum of these runs.
    #[arg(long, default_value_t = 3)]
    pub runs: usize,
}

#[derive(Subcommand)]
pub enum PluginsAction {
    /// Copy a local executable into ~/.blink/plugins as `blink-<name>`.
    /// There's no remote plugin registry — this only installs a file you
    /// already have.
    Install {
        /// Path to the plugin executable.
        source: PathBuf,
        /// Name to install it under (defaults to the source file's stem).
        #[arg(long)]
        name: Option<String>,
    },
}
