# ⚡ Blink

**A Rust-powered developer acceleration toolkit.**

> Blink removes friction between writing code and running software.

[![CI](https://github.com/martin-k-m/blink/actions/workflows/ci.yml/badge.svg)](https://github.com/martin-k-m/blink/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

Blink detects what kind of project you're in, reports on its dependency
health with a documented (not invented) scoring system, serves it locally
with live reload, checks for known vulnerabilities, and gives you an
interactive terminal dashboard — all from one small, fast binary you
install with `npm install -g blink-cli`.

New here? Start with [`docs/getting-started.md`](docs/getting-started.md).

## Features

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
- **Rule-based recommendations** — grouped into Performance/Maintenance/
  Security, each rule tied to one concrete, checkable fact.
- **A live dev server and dashboard** — an async static file server with
  a debounced file watcher, plus an interactive `ratatui` terminal UI
  that refreshes automatically on file changes.
- **CI integration** — `blink ci` with pipeline-friendly exit codes
  (0 pass / 1 warnings / 2 failure).
- **A real plugin system** — subprocess-based, `cargo`/`git`-style: any
  `blink-<name>` executable becomes `blink <name>`. No unsafe dynamic
  loading, no fake registry. See [`docs/plugins.md`](docs/plugins.md).
- **Multiple export formats** — JSON, Markdown, and self-contained HTML
  reports (`blink report`).
- **Two build caches** — a project-local content-hash cache (`blink
  build`) and a global per-user analysis cache that makes repeated
  `analyze`/`deps`/`health`/`ci` runs against an unchanged project fast.
  `blink benchmark` measures the real difference.

See [`docs/architecture.md`](docs/architecture.md) for how the nine
crates in this workspace fit together, [`docs/analysis.md`](docs/analysis.md)
for exactly what the analyzer measures (and where it's a heuristic), and
[`docs/roadmap.md`](docs/roadmap.md) for what's shipped versus planned.

## Installation

```sh
npm install -g blink-cli
blink --version
```

This downloads the `blink` binary matching your platform from a
[GitHub release](https://github.com/martin-k-m/blink/releases), verifies
its checksum, and puts a `blink` command on your `PATH`. Supported:
macOS (x64, arm64), Linux (x64, arm64), Windows (x64) — exactly the
targets built by [`.github/workflows/release.yml`](.github/workflows/release.yml).

Prefer building from source, or on an unsupported platform?

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

### Every other command

| Command | Purpose |
| --- | --- |
| `blink init` | Create a `blink.toml`. |
| `blink deps` | Dependency counts, largest packages, issues. |
| `blink health` | Health score with sub-scores. |
| `blink recommend` | Categorized, rule-based recommendations. |
| `blink run` | Dev server + file watcher. |
| `blink watch` | Analysis-only live reload (no dev server). |
| `blink build` | Cache-aware file scan. |
| `blink ci` | Analysis with pipeline exit codes. |
| `blink security` | OSV.dev vulnerability check. |
| `blink report` | Export JSON/Markdown/HTML. |
| `blink plugins` | List/install plugins. |
| `blink benchmark` | Measure Blink's own performance. |
| `blink dashboard` | Interactive terminal UI. |

Full reference with every flag: [`docs/cli.md`](docs/cli.md).

## Configuration

```toml
# blink.toml
[project]
name = "my-app"
ignore = ["vendor"]

[server]
port = 3000

[optimization]
cache = true
analyze = true
```

Full reference: [`docs/configuration.md`](docs/configuration.md).

## Architecture

```
User → npm package (packages/blink-cli) → downloads binary → blink-cli
                                                                  │
        ┌──────────┬───────────┬─────────┬──────────┬───────────┼───────────┬────────────┐
        ▼          ▼           ▼         ▼          ▼           ▼           ▼            ▼
   blink-core  blink-      blink-cache blink-    blink-       blink-      blink-plugin  blink-dashboard
   detection   analyzer    build +     server    report       parser      subprocess    ratatui TUI
   + config    dependency  analysis    dev+watch formatting   manifest/   plugins
               health      caching                            lockfile
```

Details, data flow, and the reasoning behind each design decision live in
[`docs/architecture.md`](docs/architecture.md).

## Roadmap

- **v0.1–v0.2 (shipped):** CLI, project detection, analyzer, dev server,
  build caching, health score, JSON export.
- **v0.3–v0.4 (shipped):** npm distribution, config ignore list, expanded
  detection, `deps`/`health`/`recommend`/`watch`/`ci`/`security`/`report`
  commands, a global analysis cache, a real plugin system, an interactive
  dashboard.
- **v0.5 (planned):** a real build/optimization pipeline — `blink build`
  is currently cache bookkeeping, not a bundler.
- **v0.6 (planned):** VS Code extension, a plugin registry.

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
