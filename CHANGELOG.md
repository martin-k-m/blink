# Changelog

All notable changes to Blink are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and Blink aims to
follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Versioning

Blink is **early alpha**. `0.1.0` is the first public release; expect rough
edges and breaking changes before `1.0`. The `v0.1`–`v0.5` labels used
elsewhere in the docs are internal *development milestones* (chunks of work
merged to `main`), not release versions — the released version is `0.1.0`.

> Maintainer note: an early build was briefly published to npm as `0.5.0`;
> that number overstated maturity and the project reset to `0.1.0` alpha.
> The code is the same. If you installed `0.5.0`, `0.1.0` is the successor.

## [0.1.0] — 2026-07-16 — early alpha

The first public release. Everything Blink does today, shipped as an alpha.

### Added
- **Intelligence:** `inspect`, `optimize`, `duplicates`, `doctor`,
  `filesystem`, `config-audit`, `docs`.
- **Indexing:** an incremental file/symbol index (`.blink/index.json`)
  behind `index`, `status`, `search`, `symbols`, `hotspots`, `timeline`.
- **Workflow:** `tasks`, `task`, `profile`, `clean`, `env`, `check`,
  `setup`, `completions`.
- **Configuration:** `.bnk` as an alternate filename for the `blink.toml`
  schema, with `[commands]`, `[index]`, `[profiles]`, `[scan]`, `[report]`,
  and `[plugins.*]` tables, plus `blink config check`.
- Two new crates: `blink-index` and `blink-workflow`.
- `docs/FEATURE_AUDIT.md`, actionable error messages, a grouped
  `blink --help`, and `node_project` + `monorepo` test fixtures.

### Changed
- Refreshed the README and docs to the shipped surface (eleven crates, the
  full command set, `.bnk`).

### Distribution
- Published to npm as **`@martin-k-m/blink`**, with a public GitHub release
  carrying checksummed binaries for macOS (x64/arm64), Linux (x64/arm64),
  and Windows (x64). Install with `npm install -g @martin-k-m/blink`. (The
  unscoped `blink-cli` on npm is an unrelated package.)

### Notes
- `blink run` stayed the dev server; task-running is `blink task`.
- Deferred deliberately: a remote plugin registry and `blink self update`.

---

## Pre-release development milestones

The entries below (`v0.1`–`v0.4`) are internal development milestones that
built up to the `0.1.0` alpha above — they were never published as separate
releases. They're kept for history.

## [v0.4] — Intelligent platform

### Added
- `deps`, `health`, `recommend`, `watch`, `ci`, `security`, `report`.
- A global per-user analysis cache and `blink benchmark`.
- A subprocess plugin system (`blink <name>` → `blink-<name>`).
- An interactive `ratatui` dashboard (`blink dashboard`).

## [v0.3] — Installable CLI

### Added
- `packages/blink-cli`: npm distribution that downloads and checksum-verifies
  the platform binary.
- `scan --verbose`, `[project].ignore`, expanded framework/workspace
  detection.
- A GitHub Actions release workflow (linux/macos/windows targets).

## [v0.2] — Intelligent project analysis

### Added
- Split `blink-parser` and `blink-report` out of core/analyzer.
- Direct/transitive counts, largest-package ranking, a documented health
  score, `analyze --json`, polished tables and spinners, analyzer fixtures.

## [v0.1] — MVP + developer loop

### Added
- `init`, `scan`, `analyze`, `run` (dev server + watcher), `build`
  (content-hash cache) for Rust, TypeScript/JavaScript, and Python.
