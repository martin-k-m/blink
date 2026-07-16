# Website Data

Structured reference for building a marketing/docs site for Blink. Every
item under "Current" is shipped and working as of this doc; nothing here
is aspirational — future work is listed separately under "Roadmap" and
should stay clearly labeled as such wherever this content is reused.

## Tagline

> Blink removes friction between writing code and running software.

## Current features

- **Project detection** — Rust, TypeScript/JavaScript, and Python
  projects, detected from their manifest files. Framework detection for
  React, Next.js, Vue, Svelte, and Vite (Cargo workspaces and Python
  virtualenvs are also detected).
- **Dependency analysis** — direct/transitive counts, unused-dependency
  detection, duplicate-version detection, largest-installed-package
  ranking, and opt-in outdated-package / vulnerability checks (OSV.dev).
- **A documented health score** — three independently measurable
  sub-scores (dependencies, configuration, code organization), not an
  invented number.
- **Rule-based recommendations** — grouped into Performance/Maintenance/
  Security, each rule tied to one concrete, checkable fact.
- **A live dev server** — async static file server with a debounced file
  watcher; saving a file triggers a dependency-graph rebuild.
- **An interactive terminal dashboard** — `ratatui`-based, live-refreshing
  on file changes.
- **A real plugin system** — subprocess-based, `cargo`/`git`-style
  (`blink <name>` runs `blink-<name>` if installed).
- **CI integration** — `blink ci` with pipeline-friendly exit codes.
- **Multiple export formats** — JSON, Markdown, and self-contained HTML
  reports.
- **A build cache** — content-hash based, both project-local
  (`blink build`) and a global per-user cache that speeds up repeated
  analysis.
- **Self-benchmarking** — `blink benchmark` measures Blink's own startup,
  scan, and analysis performance on real, current-run data.

## Installation

```sh
npm install -g blink-cli
blink --version
```

Or build from source:

```sh
git clone https://github.com/martin-k-m/blink.git
cd blink
cargo install --path crates/blink-cli
```

## CLI commands

All working today:

| Command | Purpose |
| --- | --- |
| `blink init` | Create a `blink.toml`. |
| `blink scan` | Detect the project and report what was found. |
| `blink analyze` | Full dependency health report. |
| `blink deps` | Dependency counts, largest packages, issues. |
| `blink health` | Health score with sub-scores. |
| `blink recommend` | Categorized, rule-based recommendations. |
| `blink run` | Dev server + file watcher. |
| `blink watch` | File-watching, analysis-only (no dev server). |
| `blink build` | Cache-aware file scan. |
| `blink ci` | Analysis with pipeline exit codes. |
| `blink security` | OSV.dev vulnerability check. |
| `blink report` | Export JSON/Markdown/HTML. |
| `blink plugins` | List/install plugins. |
| `blink benchmark` | Measure Blink's own performance. |
| `blink dashboard` | Interactive terminal UI. |

Full reference: [`docs/cli.md`](cli.md).

## Architecture

Nine Rust crates in one Cargo workspace, plus an npm distribution package:

`blink-parser` (manifest/lockfile parsing) → `blink-core` (project
detection, config) → `blink-analyzer` (dependency health) → `blink-report`
(output formatting) → `blink-cli` (commands, terminal UI), with
`blink-cache` (build + analysis caching), `blink-server` (dev server,
file watching), `blink-plugin` (subprocess plugins), and `blink-dashboard`
(the terminal UI) alongside. `packages/blink-cli` is the npm package that
distributes the compiled binary.

Full breakdown: [`docs/architecture.md`](architecture.md).

## Supported platforms

Binaries are built and published for:

- macOS (x64, arm64)
- Linux (x64, arm64)
- Windows (x64)

These are exactly the targets in `.github/workflows/release.yml`'s build
matrix — if a platform isn't listed there, there's no prebuilt binary for
it yet, and `npm install` will say so rather than silently failing.

## Roadmap (not yet built)

- A real build/bundling pipeline (`blink build` is currently cache
  bookkeeping, not a bundler).
- AST-aware unused-dependency detection (today's is a source-text scan).
- A VS Code extension.
- A plugin *registry* (today, `blink plugins install` only copies a local
  file — there's nothing to browse or fetch by name).

Full detail: [`docs/roadmap.md`](roadmap.md).
