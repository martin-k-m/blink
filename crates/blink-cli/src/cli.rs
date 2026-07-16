use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "blink",
    version,
    about = "Understand and optimize any project — fast.",
    long_about = "\u{26a1} Blink — understand and optimize any project, fast.\n\
\n\
New to a repo? Start here:\n\
  blink inspect     What is this project, how to run it, where to start\n\
  blink doctor      Check your environment can build it (runtimes, tools)\n\
  blink setup       Install dependencies and prepare the project\n\
\n\
Understand it:\n\
  analyze  deps  health  optimize  security   dependency & quality analysis\n\
  index  search  symbols  hotspots  timeline   fast, indexed code intelligence\n\
\n\
Work in it:\n\
  run  watch  build      dev server, live analysis, build cache\n\
  tasks  task  profile   discover and run project commands\n\
  check  clean  env       validate, clean caches, manage .env\n\
\n\
Report on it:\n\
  report  docs  duplicates  filesystem  config-audit  dashboard\n\
\n\
Configuration lives in blink.toml (or .bnk). Validate it with\n\
`blink config check`. Run `blink <command> --help` for details and\n\
examples on any command."
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

    // --- Phase 5: universal project intelligence ---
    /// One-screen overview: what this project is, how to run it, where to start.
    Inspect(InspectArgs),
    /// Rule-based optimization report with a score and concrete suggestions.
    Optimize(FormatArgs),
    /// Find files with identical contents (wasted space).
    Duplicates(FormatArgs),
    /// Check that this project can be developed here (runtimes, tools, env vars).
    Doctor(FormatArgs),
    /// Show where the repository's bytes live (source vs. regenerable weight).
    Filesystem(FormatArgs),
    /// Audit which standard project-config files are present.
    ConfigAudit(FormatArgs),
    /// Generate a Markdown project summary from measured facts.
    Docs(DocsArgs),

    // --- Phase 7: intelligent project indexing ---
    /// Build or incrementally refresh the on-disk project index.
    Index(IndexArgs),
    /// Show index status: files, symbols, size, last update.
    Status(FormatArgs),
    /// Search indexed file paths (or symbols with --symbols).
    Search(SearchArgs),
    /// List indexed symbols (functions, types, ...), optionally filtered.
    Symbols(SymbolsArgs),
    /// Show the largest and most-frequently-changed files.
    Hotspots(HotspotsArgs),
    /// Show recent development activity from local Git history.
    Timeline(TimelineArgs),

    // --- Phase 6/8: daily workflow engine ---
    /// List tasks discovered from package.json, Makefile, justfile, Cargo, config.
    Tasks(FormatArgs),
    /// Run a discovered task by name (see `blink tasks`).
    Task(TaskArgs),
    /// Run a named command sequence from `[profiles]` in blink.toml/.bnk.
    Profile(ProfileArgs),
    /// Remove regenerable cache/build directories (asks first).
    Clean(CleanArgs),
    /// Compare `.env` against `.env.example` (variable names only).
    Env(FormatArgs),
    /// Run the project's real local checks (format, lint, tests).
    Check(CheckArgs),
    /// Detect and install the project's dependencies (asks first).
    Setup(SetupArgs),
    /// Generate a shell completion script (bash, zsh, fish, powershell, elvish).
    Completions(CompletionsArgs),
    /// Work with Blink's own configuration (`blink config check`).
    Config(ConfigArgs),
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

/// Shared args for commands that take only a directory and a `--json` flag.
#[derive(Args)]
pub struct FormatArgs {
    /// Directory to operate on.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Print machine-readable JSON instead of formatted terminal output.
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct InspectArgs {
    /// Directory to inspect.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Print machine-readable JSON instead of formatted terminal output.
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct DocsArgs {
    /// Directory to document.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Write the generated Markdown to this file instead of stdout.
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

#[derive(Args)]
pub struct IndexArgs {
    /// Directory to index.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Rebuild the index from scratch instead of incrementally refreshing.
    #[arg(long)]
    pub rebuild: bool,
}

#[derive(Args)]
pub struct SearchArgs {
    /// Text to search for (matched case-insensitively).
    pub query: String,

    /// Directory to search.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Search symbol names instead of file paths.
    #[arg(long)]
    pub symbols: bool,

    /// Print machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct SymbolsArgs {
    /// Directory to read symbols from.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Only show symbols whose name contains this text (case-insensitive).
    #[arg(long)]
    pub filter: Option<String>,

    /// Print machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct HotspotsArgs {
    /// Directory to analyze.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// How many entries to show in each list.
    #[arg(long, default_value_t = 10)]
    pub limit: usize,

    /// Print machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct TimelineArgs {
    /// Directory to analyze.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// How many recent items to show.
    #[arg(long, default_value_t = 15)]
    pub limit: usize,
}

#[derive(Args)]
pub struct TaskArgs {
    /// Name of the task to run (see `blink tasks`).
    pub name: String,

    /// Project directory.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Print the command that would run without executing it.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args)]
pub struct ProfileArgs {
    /// Name of the profile to run (see `[profiles]` in blink.toml/.bnk).
    pub name: String,

    /// Project directory.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Print the commands that would run without executing them.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args)]
pub struct CleanArgs {
    /// Project directory.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Show what would be removed without deleting anything.
    #[arg(long)]
    pub dry_run: bool,

    /// Also remove heavy artifacts that need a reinstall/recompile to restore
    /// (`target`, `node_modules`, virtualenvs).
    #[arg(long)]
    pub all: bool,

    /// Skip the confirmation prompt.
    #[arg(short = 'y', long)]
    pub yes: bool,
}

#[derive(Args)]
pub struct CheckArgs {
    /// Project directory.
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(Args)]
pub struct SetupArgs {
    /// Project directory.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Run the setup steps without asking for confirmation.
    #[arg(short = 'y', long)]
    pub yes: bool,
}

#[derive(Args)]
pub struct CompletionsArgs {
    /// Shell to generate a completion script for.
    #[arg(value_enum)]
    pub shell: clap_complete::Shell,
}

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Validate the project's blink.toml/.bnk and report issues.
    Check {
        /// Project directory.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
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
