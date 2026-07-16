# Phases 5–8: Future Vision (NOT STARTED)

**Status: none of this is implemented.** This file preserves the raw
planning specs the project owner wrote for phases beyond v0.4 (what this
repo calls "Phase 3+4" in `docs/roadmap.md`), so a future session — human
or agent — has the full original context instead of a lossy summary.

Do not begin implementing any of this without explicit, current
instruction from the project owner. When that instruction comes, revisit
scope with them the way earlier phases in this project were scoped
(these documents are ambitious asks; prior phases were delivered by
triaging aggressively, being explicit about what's skipped and why, and
verifying every claim against real measured behavior rather than
plausible-sounding output — see `docs/architecture.md`'s "Design
decisions worth knowing about" and `docs/analysis.md` for the standard
this codebase holds itself to).

Each phase below is reproduced close to verbatim from the source
messages. Command names, file formats (e.g. `.bnk`), and specific
numbers are the owner's original proposals, not vetted decisions — e.g.
Phase 8 itself says to verify `.bnk` doesn't collide with existing
tooling before adopting it.

---

## Phase 5 — Universal Project Intelligence + Workflow Optimization

### Vision

Blink should become the first tool a developer runs after cloning any
repository. It should answer:

- What is this project?
- How do I run it?
- Where do I start?
- What's slowing development down?
- What can I improve?

Blink is NOT a replacement for Cargo, npm, Bun, or Vite. Blink sits
alongside them and makes developers more productive.

The goal: reduce onboarding time, reduce debugging time, reduce wasted
work.

### Pillar 1 — Universal Project Intelligence

(Keep all Phase 5 features here)

- `blink inspect`
- `blink map`
- `blink graph`
- `blink summary`
- repository comparison
- architecture detection
- framework detection
- project statistics
- Git intelligence
- documentation generation

### Pillar 2 — Workflow Optimization

Create a new optimization engine. The purpose is to identify
inefficiencies and suggest improvements based on measurable facts. Never
invent recommendations.

### New command: `blink optimize`

Example:

```
⚡ Blink Optimization Report

Project Score
91/100

Build Performance
✓ Good

Project Structure
⚠ src/utils contains 540 files

Dependencies
⚠ 17 unused dependencies

Tests
⚠ No integration tests detected

Documentation
⚠ Missing CONTRIBUTING.md

Configuration
✓ Good

Suggestions
→ Split src/utils into modules
→ Remove unused packages
→ Enable dependency caching
→ Add integration tests
```

Every recommendation must be backed by a concrete rule.

### Duplicate File Detection — `blink duplicates`

Detect:
- duplicate filenames
- duplicate assets
- duplicate source files (hash comparison)

Report: wasted space, possible cleanup.

### Large Directory Detection

Automatically detect: oversized directories, deeply nested folders,
unusually large modules. Suggest refactoring only when thresholds are
exceeded.

### Build Bottleneck Detection

Analyze: package count, workspace layout, dependency fan-out, generated
files, ignored directories. Suggest improvements based on detected
conditions. **Do NOT claim build speed improvements without
measurement.**

### Configuration Audit — `blink config-audit`

Check for common project configuration files, e.g.:

```
✓ README.md
✓ LICENSE
⚠ CONTRIBUTING.md missing
⚠ .editorconfig missing
⚠ .gitignore missing
⚠ CI configuration missing
```

Provide recommendations with explanations.

(Note: `blink health`'s configuration sub-score already covers a subset
of this — blink.toml/lockfile/.gitignore/README presence. A dedicated
`config-audit` command would need to either supersede that or clearly
differentiate scope.)

### Dependency Audit

Expand dependency analysis. Detect: unused dependencies, duplicate
versions, outdated dependencies, oversized packages, unnecessary
transitive dependencies (when determinable). Provide clear reports.

(Note: unused/duplicate/outdated/oversized are already implemented in
`blink-analyzer` — see `docs/analysis.md`. "Unnecessary transitive
dependencies" is the new ask here and is a much harder problem requiring
real dependency-graph reasoning, not just counting.)

### File System Optimization — `blink filesystem`

Analyze: ignored directories, binary files, generated files, cache
folders, repository size. Help developers understand where storage is
being used.

### Monorepo Intelligence

Support: Cargo workspaces, pnpm workspaces, npm workspaces, Turborepo,
Nx. Show: workspace layout, packages, shared libraries, dependency
relationships.

(Note: Cargo `[workspace]` detection already exists as a boolean flag on
`Project`; this asks for much deeper multi-package awareness.)

### Startup Analysis — `blink doctor`

Purpose: verify that the project can be developed successfully. Check:
required runtimes installed, package manager availability, compiler
availability, missing environment variables (**names only, never
values**), missing configuration files.

```
✓ Rust installed
✓ Node installed
⚠ Docker missing
✓ Git available
✓ Cargo available
```

### Smart Ignore Engine

Automatically ignore: `node_modules`, `target`, `dist`, `build`,
`coverage`, `.git`, `.cache`. Allow users to customize through
`blink.toml`.

(Note: this already exists — `blink_core::DEFAULT_IGNORED_DIRS` +
`[project].ignore`, documented in `docs/configuration.md`. `coverage`
isn't currently in the built-in list; that'd be a one-line addition if
picked up.)

### Parallel Analysis

Improve internal performance. Use: rayon where appropriate, async I/O
where beneficial, incremental directory traversal, caching, efficient
hashing. Avoid unnecessary filesystem reads. **Benchmark every
optimization.**

### Incremental Engine

Blink should remember previous analyses. If only one file changed, do
not rescan the entire project — only update affected data. This should
significantly improve repeat runs.

(Note: the global analysis cache added in v0.4 already does
whole-project-result caching keyed by a full content-hash snapshot —
a cache hit skips recomputation entirely, but any single file change
invalidates the whole cached result and triggers a full recompute. True
incremental *partial* updates — recomputing only what a specific changed
file affects — is a materially different and harder engine; this is
likely what Phase 7's indexing engine is meant to enable.)

### Machine-Readable Output

Every command should support `--json`. Where appropriate, also support
`--markdown` and `--html`. Enables integration with CI pipelines,
scripts, IDEs, dashboards, other tooling.

(Note: `blink analyze`, `health`, `report`, `security` already support
`--json`; `report` already supports `--markdown`/`--html`. Extending
this pattern to newer commands proposed here would follow the existing
convention.)

### Better Documentation — `blink docs`

Automatically generate project summaries: architecture overview,
detected technologies, project statistics, dependency summary. Output
should be suitable as a starting point for documentation, while clearly
marking generated sections for review.

### Plugin System (expansion)

Allow plugins to register: technology detectors, analyzers, exporters,
optimization rules. Provide a documented stable API.

(Note: today's `blink-plugin` is deliberately subprocess-only — a
plugin is an external executable Blink execs, with no in-process
registration API. "Register a detector/analyzer" as described here
implies either an in-process plugin ABI (the dynamic-loading approach
`blink-plugin`'s docs explicitly chose not to build, for real safety/
ABI-stability reasons) or a convention for subprocess plugins to feed
structured data back into Blink's pipeline. Worth resolving deliberately
before implementing, not defaulting to whichever is easiest.)

### Performance Goals

Optimize for responsiveness. Targets (measure and report actual results
rather than hard-coding them): fast startup, efficient incremental
analysis, low memory usage, scalable to large repositories. Use
benchmarking to validate improvements.

### Final Goal

Blink should become the fastest way to: understand a repository,
diagnose common project issues, discover optimization opportunities,
generate useful reports, onboard new developers, integrate into existing
development workflows — without replacing the tools developers already
use.

---

## Phase 6 — Daily Developer Workflow Engine

### Objective

Blink is now a mature project intelligence platform (per Phase 5). Phase
6 transforms Blink into something developers use every day: if Blink is
installed, developers should naturally reach for it dozens of times per
week. Remove friction from common development tasks while remaining
lightweight, fast, and transparent. **Never hide what Blink is doing.**
Always show the underlying commands when appropriate and allow users to
opt out.

### Philosophy

Blink should not replace Git, Cargo, npm, pnpm, Bun, Python, or Docker.
Instead, Blink should orchestrate them intelligently — a universal
workflow engine.

### Step 1 — Universal Task Runner: `blink run`

(Note: `blink run` is already Blink's dev-server command as of v0.1.
This step redefines `blink run` as a task runner instead/also — a
naming collision with existing behavior that needs to be resolved
deliberately, not silently overloaded.)

Automatically detect the project and run its natural start command:

- Rust: `cargo run`
- Node: `npm run dev`
- pnpm: `pnpm dev`
- Python: `python main.py`
- Go: `go run .`

If multiple choices exist, prompt the user. Support `blink run build`,
`blink run test`, `blink run lint`, `blink run deploy` — automatically
mapping common script names.

### Step 2 — Smart Project Setup: `blink setup`

Inspect the project and automatically: detect runtimes, install
dependencies, validate configuration, copy `.env.example` when
appropriate, install Git hooks if configured, verify required tools.
Confirm before execution unless `--yes` is provided.

### Step 3 — Unified Project Check: `blink check`

Run every supported project validation: formatting, linting, tests,
dependency audit, project health, security. Return appropriate exit
codes. Perfect for CI.

(Note: `blink ci` already exists with a 0/1/2 exit-code contract for a
similar purpose — `check` vs `ci` need a clear differentiation, or one
should supersede the other.)

### Step 4 — Smart Clean: `blink clean`

Automatically detect cache folders: Rust `target/`; Node
`node_modules/`, `dist/`, `.next/`; Python `__pycache__/`, `.venv/`.
Support `blink clean --dry-run` and `blink clean --all`.

### Step 5 — Environment Validation: `blink doctor`

(Same command name as Phase 5's `blink doctor` — the two specs describe
overlapping but not identical scope; reconcile into one command.)

Inspect: runtimes, SDKs, compilers, Docker, Git, package managers,
required environment variables. Report `✓ Installed` / `⚠ Missing` /
`❌ Broken`. **Never expose secret values.**

### Step 6 — Environment Variable Manager: `blink env`

Detect `.env` and `.env.example`; compare required variables. Report
configured / missing / unused. Support `blink env validate` and
`blink env create`.

### Step 7 — Dependency Maintenance: `blink update`

Support Cargo, npm, pnpm, Yarn, Bun, Python. Support `--check` and
`--apply`. **Never update automatically without confirmation.**

### Step 8 — Task Discovery: `blink tasks`

Automatically discover `package.json` scripts, `Makefile`, `Taskfile`,
`justfile`, Cargo aliases. Display task / description / underlying
command. Support `blink run <task>`.

### Step 9 — Smart Search: `blink search`

E.g. `blink search TODO`, `blink search User`, `blink search Config`.
Use fast indexed searching.

(Note: also specified, with more index-engine detail, in Phase 7.)

### Step 10 — Session Memory

Store recent projects, recent commands, favorite projects. Commands:
`blink recent`, `blink history`, `blink favorites`.

### Step 11 — Shell Completion

Generate completion scripts for bash, zsh, fish, PowerShell.
Automatically install if requested.

### Step 12 — Self Update: `blink self update`

Check latest release, download, verify checksum, replace binary.
Support rollback if update fails.

### Step 13 — Performance Improvements

Optimize startup time, cache performance, filesystem traversal,
parallel analysis, memory usage. Benchmark improvements. **Never
hardcode benchmark numbers.**

### Step 14 — Plugin Registry

Expand the plugin system: `blink plugin search`, `blink plugin
install`, `blink plugin remove`, `blink plugin update`. Plugins may add
analyzers, detectors, exporters, integrations. Provide a stable plugin
API.

(Note: today's plugin system explicitly has no registry — see
`docs/plugins.md`. A real `search`/`install <name>` implies standing up
actual registry infrastructure, not just CLI surface.)

### Step 15 — Documentation

Update README, CLI Reference, Architecture, Examples, Plugin Guide,
Migration Guide. Every command must have examples.

### Step 16 — Final Polish

Ensure consistent CLI output, excellent error messages, fast startup,
responsive execution, comprehensive tests, CI passing, cross-platform
support.

### Definition of Done

A new developer should be able to run `blink setup`, `blink inspect`,
`blink run`, `blink check`, `blink doctor` without reading documentation
and immediately understand what each command does. Focus: developer
productivity, reliability, an exceptional CLI experience.

### A standout feature suggestion: `blink explain`

Example: `blink explain src/auth/login.rs` would output:

- What this module does
- Which files depend on it
- Which configuration affects it
- Where it's used
- Entry points
- Related tests
- Related routes

This would require static analysis (and potentially optional LLM
integration later). Flagged by the owner as a differentiator worth
prioritizing in some future phase, not a specific phase's required
deliverable.

---

## Phase 7 — Intelligent Project Indexing

### Objective

Create a high-performance indexing engine that powers every Blink
command. Blink should stop rescanning entire projects on every
execution. Instead: index once, update incrementally, answer questions
instantly. The index becomes Blink's core advantage.

### `blink index`

Scan the repository, build a local index, cache metadata, store file
relationships, detect changes. Use incremental updates whenever
possible.

### Fast Search: `blink search`

Search filenames, directories, symbols (where supported), configuration
files, dependencies. Results should appear nearly instantly after
indexing.

### File Metadata Database

Track file paths, sizes, hashes, modification times, language,
framework ownership. Avoid reparsing unchanged files.

### Symbol Discovery

For supported languages, detect functions, structs, classes, enums,
interfaces. `blink symbols` outputs matched symbol names (example given:
`User`, `Project`, `Config`, `Analyzer`, `Workspace`).

### Dependency Cross-References

Track internal module relationships, workspace packages, project
boundaries. Makes `blink graph` significantly faster.

### Project Timeline: `blink timeline`

Show recently modified files, most active directories, recent growth.
Based entirely on local Git history.

### Hot File Detection: `blink hotspots`

Identify largest files, most frequently changed files, directories with
concentrated changes. Help developers identify maintenance risks.

### Workspace Awareness

Detect Cargo/pnpm/npm workspaces, Turborepo, Nx. Allow commands to
target a specific package: `blink inspect frontend`, `blink inspect
backend`.

### Saved Queries

```sh
blink query save backend-api "src/api/**"
blink query run backend-api
```

### Incremental Updates

Whenever Blink runs, only re-index files that changed. Never rebuild
the entire index unless necessary.

### Performance

Optimize parallel indexing, memory usage, filesystem traversal, cache
invalidation. Benchmark every improvement.

### CLI Improvements: `blink status`

Display index status, indexed files, last update, cache size, project
size.

### Definition of Done

A fast local index powers `inspect`, `search`, `graph`, `stats`,
`optimize`, `report` with significantly faster repeat execution. The
indexing engine becomes the foundation for future Blink capabilities.

### Why this phase, per the owner's own framing

"Instead of adding more surface area, you're making everything Blink
already does better: faster repeated runs, better search, better
project understanding, less filesystem work, better scalability on
large repositories. That kind of engineering work is also something
recruiters appreciate because it demonstrates performance-oriented
systems design rather than just accumulating features... I'd also
encourage measuring and publishing real benchmarks (startup time, cold
scan vs. warm scan, indexing time on representative repositories) rather
than adding many more commands. Real performance data and a polished
implementation often make a stronger impression than a larger command
list."

---

## Phase 8 — Project Intelligence Configuration System (`.bnk`)

### Objective

Create a Blink-native project configuration system: an optional project
config file. The goal: customize Blink's behavior while keeping Blink
fast, lightweight, optional, and language-agnostic.

### Before implementation — required verification

1. Verify `.bnk` is not already used by a significant programming
   language or common tooling.
2. If a conflict exists, fall back to `blink.toml` (**note: this repo
   already uses `blink.toml` as of v0.1** — see
   `docs/configuration.md`) or `.blink` (**note: `.blink/` is already
   used as of v0.1 for the project-local build cache directory** — see
   `docs/analysis.md`'s cache section). Both proposed fallback names are
   already taken by this project's existing, shipped conventions. This
   needs to be resolved explicitly before picking a file name/format —
   it may mean extending `blink.toml` (already exists, already has
   `[project]`/`[server]`/`[optimization]` tables) rather than
   introducing a fourth config surface.

### Philosophy

The config file should not replace `package.json`, `Cargo.toml`,
`pyproject.toml`, or `Makefile` — it only configures Blink's own
behavior.

### Example file (owner's proposed shape, TOML)

```toml
[project]
name = "my-app"
type = "web"

[scan]
ignore = ["node_modules", "dist", ".cache"]

[commands]
dev = "npm run dev"
test = "npm test"
build = "npm run build"

[index]
enabled = true
auto_update = true

[report]
format = "markdown"
```

(Note: `[scan].ignore` overlaps with the already-shipped
`[project].ignore` in `blink.toml`. `[commands]` overlaps with Phase 6's
task-runner/task-discovery asks. Reconciling this file's shape with
`blink.toml`'s existing schema — rather than introducing a parallel,
overlapping one — is a real design decision, not just a naming choice.)

### Step 1 — Configuration Parser: `blink-config` crate

Responsibilities: load the config file, validate syntax, provide
defaults, expose a configuration API.

(Note: `blink-core::BlinkConfig` already does exactly this for
`blink.toml`. A new crate would need a clear reason to exist separately
rather than extending the existing one.)

### Step 2 — Configuration Discovery

Search current directory, parent directories, workspace roots (e.g. a
monorepo with the config at the repo root and apps in subdirectories).
Support inheritance.

### Step 3 — CLI Integration

All Blink commands should automatically respect the config: `inspect`
uses project name/ignored folders/settings; `clean` uses cleanup rules;
`run` uses custom commands; `report` uses report preferences.

### Step 4 — Custom Project Commands: `blink task`

Read `[commands]` from config, e.g. `dev`, `docs`, `deploy`. Usage:
`blink task dev`. Blink becomes a universal project command runner.

### Step 5 — Environment Profiles

```toml
[profiles.dev]
commands = ["docker compose up", "npm install"]

[profiles.test]
commands = ["npm test"]
```

Usage: `blink profile dev`.

### Step 6 — Project Templates

`blink init` creates the config file with detected defaults — e.g. on a
React project, generates `[project] type = "web"` and
`[scan] ignore = ["node_modules"]`.

(Note: `blink init` already exists and already writes `blink.toml` with
a detected project name — see `crates/blink-cli/src/commands/init.rs`.
This step would extend that existing command, not create a new one.)

### Step 7 — Configuration Validation: `blink config check`

```
⚡ Blink Configuration

✓ .bnk valid
✓ Commands detected
✓ Ignore paths valid

Warnings:
⚠ deploy command not found
```

### Step 8 — Plugin Integration

Allow plugins to define their own configuration section, e.g.:

```toml
[plugins.react]
component_paths = ["src/components"]

[plugins.rust]
workspace = true
```

Plugins should not modify Blink core.

### Step 9 — Performance Requirements

Must be extremely lightweight: parse only once per execution, cache
parsed configuration, avoid unnecessary filesystem searches, no network
access, no background processes. Startup impact should be minimal.

### Step 10 — Documentation: `docs/configuration`

Explain why the config file exists, its syntax, examples, advanced
usage.

(Note: `docs/configuration.md` already exists for `blink.toml` — this
would extend it, or need a clear reason to split.)

### Step 11 — Update Website

Add a "Blink Configuration" section ("Make Blink understand your
workflow") showing the config file example and explaining commands,
indexing preferences, project rules, plugins.

### Definition of Done

Without a config file, `blink inspect` (and everything else) works
automatically. With one, Blink becomes customized for that project:
configure analysis, define workflows, create shortcuts, customize
reports, extend plugins — while Blink remains fast and language
independent.

### Final goal, per the owner

"`.bnk` should become the signature feature that makes Blink
recognizable. A developer should see `.bnk` and immediately know: 'This
project uses Blink tooling.'"

The owner also noted the resume/positioning angle: "Designed a
Rust-based developer workflow engine with a custom project configuration
system, incremental indexing, plugin architecture, and multi-language
codebase analysis" — framed as a stronger story than "just a CLI
wrapper."
