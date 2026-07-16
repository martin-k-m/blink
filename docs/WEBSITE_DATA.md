# Website Data

Structured reference for building a marketing/docs site for Blink. The
**features and commands** under "Current" are all implemented, tested, and
working. **Distribution is not:** Blink is pre-release — no npm package, no
public release, private repository (see "Status" below). Any site built
from this data must present install/availability honestly and must **not**
tell users to run `npm install -g blink-cli` (that name is an unrelated,
deprecated package). Future work stays under "Roadmap," clearly labeled.

## Status (must be represented honestly on any site)

- **Not on npm.** The installer exists and is verified against a locally
  built binary, but nothing is published. The `blink-cli` npm name is
  taken by an unrelated deprecated tool; a real published name hasn't been
  finalized or reserved.
- **No public releases.** The release workflow is configured but no tag
  has been pushed; no GitHub Release exists.
- **Private repository.** Repo/issue/CI links don't resolve publicly yet.
- **The install path that works today** is building from source with
  Cargo. Present that as the install method; label npm/releases "planned."

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

Building from source is the only supported install path today (Blink is
pre-release — see Status):

```sh
git clone <repository-url>
cd blink
cargo install --path crates/blink-cli
blink --version
```

Planned (not yet available): a one-line `npm install -g` and prebuilt
binaries per platform. Don't advertise an `npm install` command until a
real, reserved package name is published.

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
`packages/blink-cli` is the npm package that *will* distribute the
compiled binary once Blink is published (it is not, yet).

Full breakdown: [`docs/architecture.md`](architecture.md).

## Supported platforms

The release workflow (`.github/workflows/release.yml`) is *configured* to
build binaries for these targets — but nothing has been released yet, so
there are no published binaries to download today:

- macOS (x64, arm64)
- Linux (x64, arm64)
- Windows (x64)

Until a release is tagged, building from source (above) works on any
platform with a Rust toolchain. Development is verified on Windows; the
other targets are covered by CI configuration, not a hand-run.

## Roadmap (not yet built)

- **Publishing:** a public repository, tagged cross-platform releases, and
  a published npm package under a real, reserved name — the prerequisites
  for the "install it like a real tool" story to become true.
- A real build/bundling pipeline (`blink build` is currently cache
  bookkeeping, not a bundler).
- AST-aware unused-dependency detection (today's is a source-text scan).
- A VS Code extension.
- A plugin *registry* (today, `blink plugins install` only copies a local
  file — there's nothing to browse or fetch by name).

Full detail: [`docs/roadmap.md`](roadmap.md).
