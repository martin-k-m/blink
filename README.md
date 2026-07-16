# ⚡ Blink

**A Rust-powered developer acceleration toolkit.**

> Blink removes friction between writing code and running software.

[![CI](https://github.com/martin-k-m/blink/actions/workflows/ci.yml/badge.svg)](https://github.com/martin-k-m/blink/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

Blink detects what kind of project you're in, reports on its dependency
health, serves it locally with instant reload on file changes, and tracks a
content-hash build cache — all from one small, fast binary.

## Features

- **Project detection** — recognizes Rust, TypeScript, JavaScript, and
  Python projects by their manifests (`Cargo.toml`, `package.json`,
  `requirements.txt` / `pyproject.toml`), along with the framework
  (React, Next.js, Vue, Svelte, Cargo) and package manager (npm, pnpm,
  yarn, cargo, pip) in use.
- **Dependency analysis** — a direct dependency graph, unused-dependency
  detection via source scanning, duplicate-version detection via lockfile
  parsing, large-dependency flagging, and an opt-in registry check for
  outdated packages.
- **Dev server** — an async static file server with a debounced file
  watcher; saving a file triggers a dependency-graph rebuild and cache
  invalidation in milliseconds.
- **Build cache** — SHA-256 content hashing so `blink build` can tell you
  exactly what changed since the last run instead of treating every file
  as new work.

See [`docs/architecture.md`](docs/architecture.md) for how the five crates
in this workspace fit together, and [`docs/roadmap.md`](docs/roadmap.md)
for what's shipped versus planned.

## Installation

Blink isn't published to crates.io yet. Build it from source:

```sh
git clone https://github.com/martin-k-m/blink.git
cd blink
cargo install --path crates/blink-cli
```

This installs a `blink` binary. Prebuilt binaries for Windows, macOS, and
Linux are attached to each [GitHub release](https://github.com/martin-k-m/blink/releases)
via the [release workflow](.github/workflows/release.yml).

## Usage

### `blink init`

Creates a `blink.toml` configuration for a project (the directory is
created if it doesn't already exist):

```
$ blink init my-app

⚡ Blink
  Creating project...
  ✓ Detected environment
  ✓ Created configuration
  ✓ Ready

  Project initialized.
```

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

Reports on dependency health and prints recommendations derived from what
it actually found — nothing generic:

```
$ blink analyze

⚡ Blink Analysis Report

  Dependencies
    ✓ Healthy packages: 9
    ⚠ Duplicate versions: 3
    - Outdated packages: unknown (run with --online to check)

  Performance
    Analysis time    7ms
    Build output     not built yet

  Recommendations
    → Deduplicate windows-sys (4 versions resolved: 0.48.0, 0.52.0, 0.59.0, 0.61.2)
    → Deduplicate windows-targets (2 versions resolved: 0.48.5, 0.52.6)
    → Deduplicate mio (2 versions resolved: 0.8.11, 1.2.2)
```

Pass `--online` to additionally check crates.io / the npm registry for
newer published versions. This is the only network call anywhere in
Blink, and it's off by default.

### `blink run`

Starts the dev server and file watcher:

```
$ blink run

⚡ Blink Development Server
  Starting...

  ✓ Project detected
  ✓ Dependencies loaded (12)
  ✓ Server ready

  Local             http://localhost:3000

  Ready in: 2ms

  Watching for file changes... (Ctrl+C to stop)

src/App.tsx changed
Blink:
  ✓ Updated dependency graph
  ✓ Cache invalidated for changed files
  4ms
```

### `blink build`

Runs a cache-aware build pass:

```
$ blink build

⚡ Blink Build
  Building...

  ✓ Compared 45 files against cache

  Cache
    43 unchanged
    2 changed
    0 added
    0 removed
  ✓ Cache saved

  Build complete in: 6ms
```

Every timing and count shown above is measured at run time — none of it
is hardcoded. Run these commands against this repository yourself and
you'll see your own numbers.

## Configuration

```toml
# blink.toml
[project]
name = "my-app"

[server]
port = 3000

[optimization]
cache = true
analyze = true
```

Full reference: [`docs/configuration.md`](docs/configuration.md).

## Architecture

```
                    ┌─────────────┐
                    │  blink-cli  │  clap commands, terminal UI
                    └──────┬──────┘
           ┌───────────────┼───────────────┬───────────────┐
           ▼                ▼               ▼               ▼
   ┌───────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
   │  blink-core   │ │blink-analyzer│ │ blink-cache │ │blink-server │
   │  detection +  │ │  dependency  │ │  content-   │ │  dev server │
   │  config       │ │  health      │ │  hash cache │ │  + watcher  │
   └───────────────┘ └─────────────┘ └─────────────┘ └─────────────┘
```

Details, data flow, and the reasoning behind each design decision live in
[`docs/architecture.md`](docs/architecture.md).

## Roadmap

- **Phase 1 (shipped):** CLI, project scanner/detector, analyzer reports.
- **Phase 2 (shipped):** dev server, file watching, change notifications.
- **Phase 3 (partially shipped):** build caching is done; a real
  optimization/bundling pass is not — `blink build` is currently a
  cache-aware file scanner, not a bundler.
- **Phase 4 (planned):** VS Code extension, plugin system, deeper
  suggestions.

Full detail in [`docs/roadmap.md`](docs/roadmap.md).

## Development

```sh
cargo build --workspace       # build every crate
cargo test --workspace        # unit + integration tests
cargo fmt --all -- --check    # formatting
cargo clippy --workspace --all-targets -- -D warnings
```

Integration tests live at the repository root in [`tests/`](tests) and run
against the fixture projects in [`examples/`](examples) plus the built
`blink` binary (via `assert_cmd`). Unit tests live alongside the code
they test in each crate.

## Contributing

1. Fork the repo and create a branch off `main`.
2. Make your change. Add or update tests for it.
3. Run the checks in [Development](#development) — CI runs the same ones.
4. Open a pull request describing what changed and why.

Bug reports and feature requests are welcome via
[GitHub Issues](https://github.com/martin-k-m/blink/issues).

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
