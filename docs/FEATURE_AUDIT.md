# Feature Audit

A complete, honest inventory of every Blink command and subsystem, written
for the v1.0 stabilization pass (Phase 9). For each area: what it's for,
its status, how it's used, what it depends on, and its maintenance cost.
The last two sections record what this audit decided to **keep, merge, or
cut**, and why.

Measured on the current build: **11 crates**, **195 unique transitive
dependencies**, **4.4 MB release binary**, and (on this repo)
**11 ms startup · 2 ms scan · 23 ms cold analysis · 6 ms cached** via
`blink benchmark`.

## How to read the tables

- **Status** — `stable` (shipped, tested, documented), `thin` (works, but
  intentionally shallow), or `at-risk` (candidate for change; see notes).
- **Maint. cost** — rough ongoing burden: `low` (pure function over
  existing data), `med` (owns some logic/IO), `high` (long-running
  runtime, external process, or platform-specific code).

## Commands

### Getting started

| Command | Purpose | Status | Depends on | Maint. cost |
| --- | --- | --- | --- | --- |
| `init` | Write a `blink.toml` seeded with the detected project name. | stable | core | low |
| `scan` | Detect language/framework/package-manager/counts. | stable | core | low |
| `inspect` | One-screen "what is this / how to run / where to start". | stable | core, index, workflow | med |
| `doctor` | Verify runtimes/tools/env-var *names* are present. | stable | core, workflow | med |
| `setup` | Copy `.env.example`→`.env`, install deps (asks first). | stable | core, workflow, proc | med |

### Understand (analysis & intelligence)

| Command | Purpose | Status | Depends on | Maint. cost |
| --- | --- | --- | --- | --- |
| `analyze` | Full dependency-health report + suggestions. | stable | analyzer, cache | med |
| `deps` | Focused dependency counts / largest / issues. | stable | analyzer, cache | low |
| `health` | Health score with three sub-scores. | stable | analyzer, cache | low |
| `recommend` | Findings grouped Performance/Maintenance/Security. | stable | analyzer, cache | low |
| `optimize` | Rule-based 0–100 score across six categories. | stable | analyzer, index, workflow | med |
| `security` | OSV.dev vulnerability lookup (network). | stable | analyzer | med |
| `index` | Build/refresh the incremental file+symbol index. | stable | index | med |
| `status` | Index stats (files/symbols/lines/size/languages). | stable | index | low |
| `search` | Search indexed paths, or symbols with `--symbols`. | stable | index | low |
| `symbols` | List indexed symbols, optionally filtered. | stable | index | low |
| `hotspots` | Largest files + most-changed files (Git). | stable | index, workflow | low |
| `timeline` | Recent activity from local Git history. | stable | workflow | low |

### Work (run, build, tasks)

| Command | Purpose | Status | Depends on | Maint. cost |
| --- | --- | --- | --- | --- |
| `run` | Dev server + debounced file watcher. | stable | server | high |
| `watch` | Analysis-only live reload (no server). | stable | server, analyzer | high |
| `build` | Cache-aware content-hash file scan. | thin | cache | low |
| `tasks` | Discover tasks (package.json/Make/just/Cargo/config). | stable | workflow | low |
| `task` | Run a discovered task, mirroring its exit code. | stable | workflow, proc | med |
| `profile` | Run a `[profiles]` command sequence. | stable | core, proc | low |
| `check` | Run the real local toolchain (fmt/lint/tests). | stable | core, workflow, proc | med |
| `clean` | Remove regenerable dirs (asks first). | stable | workflow, proc | med |
| `env` | Compare `.env` vs `.env.example` (names only). | stable | workflow | low |
| `ci` | Analysis with 0/1/2 pipeline exit codes. | stable | analyzer, cache | low |

### Report & inspect output

| Command | Purpose | Status | Depends on | Maint. cost |
| --- | --- | --- | --- | --- |
| `report` | Export JSON/Markdown/HTML document. | stable | report | low |
| `docs` | Generate a Markdown project summary. | stable | core, index, workflow | low |
| `duplicates` | Byte-identical file groups + reclaimable space. | stable | index, workflow | low |
| `filesystem` | Where the bytes live (source vs. regenerable). | stable | workflow | low |
| `config-audit` | Which standard project-config files exist. | stable | workflow | low |
| `config check` | Validate `blink.toml`/`.bnk`. | stable | core | low |
| `dashboard` | Interactive `ratatui` TUI. | stable | dashboard | high |
| `benchmark` | Measure Blink's own performance (real numbers). | stable | analyzer, cache | low |

### Meta

| Command | Purpose | Status | Depends on | Maint. cost |
| --- | --- | --- | --- | --- |
| `plugins` | List/install subprocess plugins. | stable | plugin | low |
| `completions` | Emit a shell completion script. | stable | clap_complete | low |

## Subsystems (crates)

| Crate | Owns | Status | Maint. cost |
| --- | --- | --- | --- |
| `blink-parser` | Manifest/lockfile *format* parsing. | stable | low |
| `blink-core` | Project detection + `blink.toml`/`.bnk` config + errors. | stable | low |
| `blink-analyzer` | Dependency graph, health, recommendations, OSV. | stable | med |
| `blink-report` | Terminal/JSON/Markdown/HTML formatting. | stable | low |
| `blink-cache` | Project-local build cache + global analysis cache. | stable | med |
| `blink-server` | Dev server + debounced file watcher. | stable | high |
| `blink-plugin` | Subprocess plugin discovery/execution. | stable | low |
| `blink-dashboard` | `ratatui` terminal UI. | stable | high |
| `blink-index` | Incremental file/symbol index. | stable | med |
| `blink-workflow` | optimize/doctor/tasks/clean/env/duplicates/git. | stable | med |

## Overlap analysis — kept deliberately

Several commands look adjacent but answer different questions. They were
**kept** rather than merged, because each has a distinct, documented job
and merging would make the survivor do too much:

- **`analyze` / `deps` / `health` / `recommend`** — all read the same
  `AnalysisReport`, but they're *focused views* (a v0.4 design decision,
  not accidental duplication): `deps` is dependency tables, `health` is
  the scored breakdown, `recommend` is categorized actions, `analyze` is
  the full report. No logic is duplicated — they share one engine.
- **`optimize` vs `recommend`/`health`** — `optimize` adds structure,
  duplicate-file, and test/docs/config checks (from the index) that the
  dependency-only commands don't have, and scores them separately.
- **`check` vs `ci`** — `check` *runs your toolchain* (fmt/lint/tests);
  `ci` runs Blink's *analysis* and returns 0/1/2 exit codes. Different
  work, different audience.
- **`filesystem` vs `clean`** — `filesystem` reports where bytes live;
  `clean` deletes regenerable ones. Report vs. action.
- **`run` vs `watch`** — `run` is a dev server *with* watching; `watch`
  is analysis-only reload with no server.
- **`doctor` / `config-audit` / `config check`** — environment tools vs.
  standard project files present vs. Blink's own config validity. Three
  different questions.

**Conclusion:** no commands were removed. The audit found volume, not
redundancy — the 1.0 fix is *organization* (a grouped `--help`, this
document, and the refreshed docs), not amputation. If the project owner
wants a smaller surface, the safest candidates to *hide from top-level
help* (not delete) would be `scan` (largely subsumed by `inspect`) and
`build` (thin until the v0.6 bundler lands) — flagged here, not acted on.

## Dependency & tech-debt notes

- **No unnecessary dependencies** were added in the v0.5 work. `rayon`
  (parallel hashing in `blink-index`), `clap_complete` (completions),
  `sha2`/`hex` (content hashing) are each used on a real path.
- **Duplicate transitive versions** exist (`crossterm` 0.28/0.29,
  `windows-sys` 0.48/0.59/0.61, `hashbrown`, `unicode-width`,
  `webpki-roots`) — all pulled in by *different* upstream crates
  (`ratatui` vs `indicatif`, etc.), not by Blink directly. They can't be
  unified without upstream alignment and aren't worth pinning around.
- **`fmt` / `clippy -D warnings` / `test`** are clean across the
  workspace; there is no dead code gated behind `#[allow]`.
- **Open, tracked separately:** GitHub Dependabot reports 4 advisories on
  `main` (1 high, 2 moderate, 1 low). These are transitive and predate
  this phase; investigating them (starting with `blink security .`) is
  its own task, noted in `docs/roadmap.md`.

## Cross-platform status

Path handling uses `PathBuf`/`std::path` throughout; the index normalizes
stored paths to `/` separators so an index is stable across OSes, and the
shell runner picks `cmd /C` on Windows and `sh -c` elsewhere. The release
matrix (`.github/workflows/release.yml`) builds macOS (x64/arm64), Linux
(x64/arm64), and Windows (x64). **This audit was run on Windows only;**
Linux/macOS behavior is covered by CI, not by a hand-run of every command
on those platforms.
