# Website Data

Structured reference for building a marketing/docs site for Blink.
Everything below is implemented, tested, and published.

## Links

- **npm:** https://www.npmjs.com/package/@martin-k-m/blink
- **Repository:** https://github.com/martin-k-m/blink
- **Install:** `npm install -g @martin-k-m/blink`

## Status

- **Published on npm** as **`@martin-k-m/blink`**, auto-released via CI on
  each version tag (with build provenance). The unscoped `blink-cli` is an
  unrelated, deprecated package — always use the scoped name.
- **Public GitHub releases** with checksummed binaries for macOS
  (x64/arm64), Linux (x64/arm64), and Windows (x64).
- **Public repository** at `github.com/martin-k-m/blink`.
- The npm package downloads its platform binary via a postinstall script;
  on npm setups that block install scripts, users allow it for this
  package or build from source.

## Tagline

> Blink removes friction between writing code and running software.

## Current features

- **Project intelligence** — `blink inspect` explains what a project is,
  how to run it, and where to start; `blink optimize` scores it against
  six concrete rule-based checks; `blink doctor` verifies the environment
  can build it.
- **A fast incremental index** — `blink index` records every file's
  hash/size/language/symbols and re-processes only what changed, powering
  instant `search`, `symbols`, `hotspots`, `timeline`, and `duplicates`.
- **A workflow engine** — discover and run project commands
  (`tasks`/`task`/`profile`), validate with the real toolchain (`check`),
  clean caches (`clean`), manage `.env` (`env`), and prepare a fresh clone
  (`setup`). Blink orchestrates existing tools; it never hides them.
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
npm install -g @martin-k-m/blink
blink --version
```

Always the scoped name `@martin-k-m/blink` (the unscoped `blink-cli` is an
unrelated package). Or build from source:

```sh
git clone https://github.com/martin-k-m/blink.git
cd blink
cargo install --path crates/blink-cli
```

## CLI commands

All working today, grouped as `blink --help` presents them:

**Get started:** `init` · `scan` · `inspect` · `doctor` · `setup`
**Understand:** `analyze` · `deps` · `health` · `recommend` · `optimize` · `security`
**Index & search:** `index` · `status` · `search` · `symbols` · `hotspots` · `timeline`
**Work in it:** `run` · `watch` · `build` · `tasks` · `task` · `profile` · `check` · `clean` · `env` · `ci`
**Report:** `report` · `docs` · `duplicates` · `filesystem` · `config-audit` · `config check` · `dashboard` · `benchmark`
**Extend:** `plugins` · `completions`

| Command | Purpose |
| --- | --- |
| `blink inspect` | What is this project, how to run it, where to start. |
| `blink optimize` | Rule-based 0–100 score with concrete suggestions. |
| `blink doctor` | Verify the environment can build the project. |
| `blink analyze` | Full dependency health report. |
| `blink index` / `search` / `symbols` | Fast, indexed code intelligence. |
| `blink tasks` / `task` | Discover and run project commands. |
| `blink check` | Run the real local toolchain (fmt/lint/tests). |
| `blink run` / `dashboard` | Dev server + watcher; interactive TUI. |

Full reference (every command and flag): [`docs/cli.md`](cli.md); a
per-command inventory is in [`docs/FEATURE_AUDIT.md`](FEATURE_AUDIT.md).

## Architecture

Eleven Rust crates in one Cargo workspace, plus an npm distribution
package:

`blink-parser` (manifest/lockfile parsing) → `blink-core` (project
detection, config) → `blink-analyzer` (dependency health) → `blink-report`
(output formatting) → `blink-cli` (commands, terminal UI), with
`blink-cache` (build + analysis caching), `blink-server` (dev server,
file watching), `blink-index` (incremental file/symbol index),
`blink-workflow` (optimize/doctor/tasks/clean/env), `blink-plugin`
(subprocess plugins), and `blink-dashboard` (the terminal UI) alongside.
`packages/blink-cli` is the npm package (`@martin-k-m/blink`) that
distributes the compiled binary.

Full breakdown: [`docs/architecture.md`](architecture.md).

## Supported platforms

Prebuilt binaries are published with each release (via
`.github/workflows/release.yml`) for:

- macOS (x64, arm64)
- Linux (x64, arm64)
- Windows (x64)

On any other platform, build from source with a Rust toolchain.
Development is verified on Windows; the other targets are built and
released by CI.

<!-- Website should present the shipped product; the forward-looking
roadmap lives in docs/roadmap.md and is intentionally not surfaced here to
avoid advertising unbuilt features. -->
