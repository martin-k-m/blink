# Changelog

All notable changes to Blink are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and Blink aims to
follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Versioning

Entries below use the milestone versions the docs reference (`v0.1`–`v0.5`).
These milestones have all been merged to `main`, but **no Git tag has been
pushed and no GitHub Release exists yet** — the first tagged release is
still pending (see [`docs/roadmap.md`](docs/roadmap.md)). Until then, the
top entry is the current state of `main`. The workspace manifest version is
independent of these milestone labels and is bumped at release time.

## [Unreleased]

Stabilization toward a 1.0 release (no new features):

### Added
- `docs/FEATURE_AUDIT.md` — a per-command and per-crate inventory.
- Actionable error messages: failures render a branded block that explains
  the cause and suggests concrete fixes.
- A grouped `blink --help` overview so the command set reads as one tool.
- `node_project` and `monorepo` test fixtures.

### Changed
- Refreshed `README.md`, `docs/WEBSITE_DATA.md`, and the docs to match the
  shipped v0.5 surface (eleven crates, the full command set, `.bnk`).
- **Corrected install/availability claims to reflect reality.** Blink is
  pre-release: not on npm, no public release, private repo. Docs now lead
  with build-from-source and label npm/releases as planned. Removed the
  `npm install -g blink-cli` instruction everywhere — that npm name is an
  unrelated, deprecated package. Added a "Status" section to the README
  and the website data.

## [v0.5] — Project intelligence, indexing & workflow engine

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

### Notes
- `blink run` stayed the dev server; task-running is `blink task`.
- Deferred deliberately (need a published release to verify against): a
  remote plugin registry and `blink self update`.

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
