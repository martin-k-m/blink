# ⚡ Blink

**A Rust-powered developer acceleration toolkit.**

> Blink removes friction between writing code and running software.

[![npm](https://img.shields.io/npm/v/@martin-k-m/blink.svg)](https://www.npmjs.com/package/@martin-k-m/blink)
[![GitHub stars](https://img.shields.io/github/stars/martin-k-m/blink)](https://github.com/martin-k-m/blink)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

Blink is the first thing to run after cloning a repo: it tells you what
the project is, how to run it, and where to start; reports on dependency
health with a documented (not invented) score; indexes the codebase for
instant search; runs the project's own tasks; and validates your
environment — all from one small, fast binary.

```sh
npm install -g @martin-k-m/blink
blink inspect
```

New here? Start with [`docs/getting-started.md`](docs/getting-started.md).

## Features

- **Project intelligence** — `blink inspect` answers "what is this / how
  do I run it / where do I start" in one screen; `blink optimize` scores
  the project against six concrete, rule-based checks; `blink doctor`
  verifies your environment can build it.
- **Project detection** — Rust, TypeScript/JavaScript, and Python
  projects, detected from their manifests. Framework detection for React,
  Next.js, Vue, Svelte, and Vite; Cargo workspaces and Python virtualenvs
  too.
- **Dependency analysis** — direct/transitive counts, unused-dependency
  detection, duplicate-version detection, largest-installed-package
  ranking (Rust and JS/TS both), and opt-in outdated-package /
  vulnerability (OSV.dev) checks.
- **A documented health score** — three independently measurable
  sub-scores (dependencies, configuration, code organization), not an
  invented number. See [`docs/analysis.md`](docs/analysis.md).
- **A fast incremental index** — `blink index` records every file's
  hash/size/language/symbols and refreshes only what changed; it powers
  instant `search`, `symbols`, `hotspots`, `timeline`, and `duplicates`.
- **A workflow engine** — discover and run project commands (`tasks`/
  `task`/`profile`), validate with the real toolchain (`check`), clean
  caches (`clean`), manage `.env` (`env`), and prepare a fresh clone
  (`setup`) — Blink orchestrates your tools, it never hides them.
- **A live dev server and dashboard** — an async static file server with
  a debounced file watcher, plus an interactive `ratatui` terminal UI
  that refreshes automatically on file changes.
- **CI integration** — `blink ci` with pipeline-friendly exit codes
  (0 pass / 1 warnings / 2 failure).
- **A real plugin system** — subprocess-based, `cargo`/`git`-style: any
  `blink-<name>` executable becomes `blink <name>`. No unsafe dynamic
  loading, no fake registry. See [`docs/plugins.md`](docs/plugins.md).
- **Machine-readable everywhere** — `--json` on the report commands, plus
  Markdown and self-contained HTML export (`blink report`) and shell
  completions (`blink completions`).
- **Caching that's measured, not claimed** — a project-local content-hash
  build cache, a global per-user analysis cache, and the incremental
  index all make repeat runs fast; `blink benchmark` measures the real
  difference on your machine.

Configuration lives in `blink.toml` — or `.bnk`, the same schema under a
signature name. See [`docs/architecture.md`](docs/architecture.md) for how
the eleven crates fit together, [`docs/analysis.md`](docs/analysis.md) for
exactly what each score measures, [`docs/FEATURE_AUDIT.md`](docs/FEATURE_AUDIT.md)
for a per-command inventory, and [`docs/roadmap.md`](docs/roadmap.md) for
what's shipped versus planned.

## Installation

```sh
npm install -g @martin-k-m/blink
blink --version
```

This downloads the `blink` binary matching your platform from the matching
[GitHub release](https://github.com/martin-k-m/blink/releases), verifies
its SHA-256 checksum, and puts a `blink` command on your `PATH`. Supported:
macOS (x64, arm64), Linux (x64, arm64), Windows (x64).

> The unscoped `blink-cli` on npm is an unrelated, deprecated package — the
> scoped **`@martin-k-m/blink`** is this tool. The binary download runs as a
> postinstall script; if your npm is configured to block install scripts,
> allow it for this package or build from source (below).

Prefer building from source, or on an unsupported platform? You'll need
[Rust](https://www.rust-lang.org/tools/install) 1.75+:

```sh
git clone https://github.com/martin-k-m/blink.git
cd blink
cargo install --path crates/blink-cli
```

## Usage

Full flag reference for every command: [`docs/cli.md`](docs/cli.md).

### `blink scan`

Detects the project in a directory:

```
$ blink scan

⚡ Blink Project Scanner
  Project           blink
  Framework         Cargo
  Language          Rust
  Package manager   cargo
  Files             45
  Dependencies      12

  Scan completed: 2ms
```

### `blink analyze`

Reports on dependency health — direct/transitive counts, a documented
health score, the largest installed packages, and recommendations derived
from what it actually found:

```
$ blink analyze

⚡ Blink Analysis

  Project           blink
  Type              Cargo (Rust)
  Files             76

  Health
    █████░░░░░ 48%

  Dependencies
  ┌────────────┬───────┐
  │ Metric     ┆ Count │
  ╞════════════╪═══════╡
  │ Direct     ┆ 5     │
  │ Transitive ┆ 211   │
  │ Healthy    ┆ 0     │
  └────────────┴───────┘

  Largest Packages
  ┌────────────┬─────────┐
  │ Package    ┆ Size    │
  ╞════════════╪═════════╡
  │ tempfile   ┆ 183.5KB │
  │ predicates ┆ 131.8KB │
  │ assert_cmd ┆ 113.9KB │
  └────────────┴─────────┘

  Potential Issues
    ⚠ Duplicate package versions detected (13)
    - Outdated packages: unknown (run with --online to check)

  Suggestions
    → Deduplicate windows-sys (4 versions resolved: 0.48.0, 0.52.0, 0.59.0, 0.61.2)
    → Deduplicate windows-targets (2 versions resolved: 0.48.5, 0.52.6)
    → Deduplicate mio (2 versions resolved: 0.8.11, 1.2.2)

  Performance
    Analysis time    18ms
    Build output     297.0MB
```

Pass `--online` to additionally check for outdated packages (`--json` for
machine-readable output). Every field is real and documented — see
[`docs/analysis.md`](docs/analysis.md).

### Every command

Run `blink --help` for the grouped overview, or `blink <command> --help`
for any one. Full reference with every flag: [`docs/cli.md`](docs/cli.md).

**Get started:** `init` · `scan` · `inspect` · `doctor` · `setup`
**Understand:** `analyze` · `deps` · `health` · `recommend` · `optimize` · `security`
**Index & search:** `index` · `status` · `search` · `symbols` · `hotspots` · `timeline`
**Work in it:** `run` · `watch` · `build` · `tasks` · `task` · `profile` · `check` · `clean` · `env` · `ci`
**Report:** `report` · `docs` · `duplicates` · `filesystem` · `config-audit` · `config check` · `dashboard` · `benchmark`
**Extend:** `plugins` · `completions`

A few highlights:

| Command | Purpose |
| --- | --- |
| `blink inspect` | What is this project, how to run it, where to start. |
| `blink optimize` | Rule-based 0–100 score with concrete suggestions. |
| `blink doctor` | Check your environment can build the project. |
| `blink search` / `symbols` | Instant indexed code search. |
| `blink task <name>` | Run a discovered project command. |
| `blink check` | Run the real local toolchain (fmt/lint/tests). |
| `blink clean` | Remove regenerable caches (asks first). |

## Configuration

```toml
# blink.toml — or name it .bnk (same schema, signature name)
[project]
name = "my-app"
ignore = ["vendor"]

[commands]           # discovered by `blink tasks`, run with `blink task <name>`
dev = "npm run dev"
test = "npm test"

[server]
port = 3000
```

Full reference (all tables, and how `.bnk` relates to `blink.toml`):
[`docs/configuration.md`](docs/configuration.md).

## Architecture

Eleven crates in one Cargo workspace, plus the npm distribution package.
`blink-parser` (formats) → `blink-core` (detection + config) →
`blink-analyzer` (health) → `blink-report` (formatting) → `blink-cli`
(commands), with `blink-cache` (caching), `blink-server` (dev server +
watch), `blink-index` (incremental file/symbol index), `blink-workflow`
(optimize/doctor/tasks/clean/env), `blink-plugin` (subprocess plugins),
and `blink-dashboard` (TUI) alongside.

Details, data flow, and the reasoning behind each design decision live in
[`docs/architecture.md`](docs/architecture.md).

## Roadmap

- **v0.1–v0.4 (shipped):** CLI, project detection, analyzer, dev server,
  build caching, health score, JSON export, npm distribution, a global
  analysis cache, a real plugin system, and an interactive dashboard.
- **v0.5 (shipped):** project intelligence (`inspect`/`optimize`/`doctor`),
  an incremental index (`index`/`search`/`symbols`/`hotspots`), a workflow
  engine (`tasks`/`task`/`check`/`clean`/`setup`/`env`), and `.bnk` config.
- **v0.6 (planned):** a real build/optimization pipeline — `blink build`
  is currently cache bookkeeping, not a bundler.
- **v0.7 (planned):** VS Code extension, a plugin registry, `self update`.

Full detail in [`docs/roadmap.md`](docs/roadmap.md).

## Development

```sh
cargo build --workspace       # build every crate
cargo test --workspace        # unit + integration tests
cargo fmt --all -- --check    # formatting
cargo clippy --workspace --all-targets -- -D warnings
```

Integration tests live at the repository root in [`tests/`](tests) and run
against the showcase projects in [`examples/`](examples), the
purpose-built fixtures in [`tests/fixtures/`](tests/fixtures) (each one
exercises a specific analyzer behavior, e.g. a hand-written `Cargo.lock`
with a deliberate duplicate version), and the built `blink` binary (via
`assert_cmd`) — including a self-contained test of the plugin
fallback-dispatch mechanism. Unit tests live alongside the code they test
in each crate; `blink-dashboard`'s rendering is tested headlessly with
`ratatui`'s `TestBackend`.

The npm package (`packages/blink-cli`) has no Rust dependencies to test,
but CI syntax-checks its scripts and runs a real `npm install` +
postinstall + shim-execution pass against a locally built binary (see the
`npm-package` job in [`.github/workflows/ci.yml`](.github/workflows/ci.yml)),
so a broken installer script fails CI the same as broken Rust code would.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the full guide. In short:

1. Fork the repo and create a branch off `main`.
2. Make your change. Add or update tests for it.
3. Run the checks in [Development](#development) — CI runs the same ones.
4. Open a pull request describing what changed and why.

Bug reports and feature requests are welcome via
[GitHub Issues](https://github.com/martin-k-m/blink/issues). Please
review our [Code of Conduct](CODE_OF_CONDUCT.md), and see
[`SECURITY.md`](SECURITY.md) for how to report a vulnerability privately.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
