# Changelog

All notable changes to Blink are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and Blink aims to
follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Versioning

**`0.5.0` is the intended first tagged release.** The workspace and the npm
package are versioned `0.5.0`; tagging `v0.5.0` triggers the release
workflow that builds the cross-platform binaries. Until that tag is pushed,
no GitHub Release or npm package exists yet (see the README's Status
section). Earlier `v0.1`–`v0.4` entries below are development milestones
that were merged to `main` but never tagged individually.

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
  full command set, `.bnk`), and corrected install claims to reflect that
  Blink is pre-release (build-from-source; npm/releases labeled planned).

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
