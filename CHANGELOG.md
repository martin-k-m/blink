# Changelog

All notable changes to Blink are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and Blink aims to
follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Versioning

Released on npm as `@martin-k-m/blink` and as GitHub releases with
cross-platform binaries. Earlier `v0.1`–`v0.4` entries below are internal
development milestones (merged to `main`, never tagged individually).

## [0.6.0] — 2026-07-16

The **context engine** — Blink's repositioning from a developer-acceleration
toolkit to a *developer context engine*: reliable, local understanding of
any codebase, for humans and their tools.

### Added
- **`blink-context`** — builds the project **context graph**, unifying
  detection, the file/symbol index, and declared dependencies into one
  serializable model, with files grouped into **areas** and file→file
  **references** resolved conservatively (TS/JS relative imports, Python
  imports, Rust `mod` trees, and cross-crate `<crate>::` paths in a Cargo
  workspace). An unresolvable import is never turned into an invented edge.
- **`blink-query`** — deterministic, local structured search over the graph
  (areas, files, symbols, dependencies, commands). Not AI, no inference.
- **`blink-export`** — serializes the graph to JSON, YAML, Markdown, and a
  Mermaid architecture graph (YAML via a small internal emitter, no new dep).
- **Commands:** `context`, `query`, `map`, `explain`, `export`. `explain`
  reports only real signals — a file's own doc, its symbols, its imports,
  and what imports it — never invented "responsibilities".
- **`[context]` config** (`enabled`, `include`) in `blink.toml`/`.bnk`,
  surfaced by `blink config check`.
- A repositioned README, docs (`docs/context-engine.md`), and a new
  self-contained landing page under `website/`.

### Changed
- Positioning across README, npm, and docs: "developer context engine" /
  "The context layer for modern software development."
- The workspace is now **fourteen** crates; the test suite is **182** tests.

## [0.5.3] — 2026-07-16

Also publishes to the GitHub Packages registry (via the built-in
`GITHUB_TOKEN`) so the package shows in the repo's "Packages" sidebar.
npmjs.org (`@martin-k-m/blink`) remains the real install source — GitHub
Packages requires auth to install. No functional changes to the tool.

## [0.5.2] — 2026-07-16

Fixes automated npm publishing. The npm publish now runs as a job *inside*
the tag-triggered release workflow (with **provenance**), instead of a
separate `on: release` workflow — the latter never fired, because a GitHub
Release created by the default `GITHUB_TOKEN` doesn't emit a `release` event
that can trigger other workflows. No functional changes to the tool.

## [0.5.1] — 2026-07-16

Tagged; binaries built and a GitHub release created — but the npm
auto-publish did *not* run (the `on: release` trigger issue fixed in
`0.5.2`), so `0.5.1` was never published to npm. Superseded by `0.5.2`.

## [0.5.0] — 2026-07-16

The first tagged release. Bundles the v0.5 feature work with the 1.0
stabilization pass.

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
- First public release. Published to npm as **`@martin-k-m/blink`**, with a
  public GitHub release (`v0.5.0`) carrying checksummed binaries for
  macOS (x64/arm64), Linux (x64/arm64), and Windows (x64). Install with
  `npm install -g @martin-k-m/blink`. (The unscoped `blink-cli` on npm is
  an unrelated package.)

### Notes
- `blink run` stayed the dev server; task-running is `blink task`.
- Deferred deliberately: a remote plugin registry and `blink self update`.

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
