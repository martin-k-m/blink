# Architecture

Blink is a Cargo workspace of eleven crates, plus an npm package that
distributes the compiled binary. Three low-level crates read and format
data with no domain knowledge of "what a project is"; the rest build up
from there. No crate depends on a "sibling" it doesn't need — `blink-cli`
is the only crate that wires everything together.

```
User
  │
  ▼
npm package (packages/blink-cli)  ── downloads a prebuilt binary, execs it
  │
  ▼
                              ┌─────────────┐
                              │  blink-cli  │  clap commands, terminal UI
                              └──────┬──────┘
   ┌────────────┬────────────┬──────┼──────┬────────────┬────────────┐
   ▼             ▼            ▼     ▼      ▼             ▼            ▼
┌─────────┐ ┌──────────┐ ┌───────┐┌──────┐┌────────┐ ┌────────┐ ┌───────────┐
│blink-   │ │blink-    │ │blink- ││blink-││blink-  │ │blink-  │ │blink-     │
│core     │ │analyzer  │ │cache  ││server││report  │ │plugin  │ │dashboard  │
│detection│ │dependency│ │content││ dev  ││format- │ │subproc │ │ratatui    │
│+ config │ │+ health  │ │-hash  ││server││ting +  │ │plugins │ │TUI        │
│         │ │          │ │cache  ││+watch││ JSON   │ │        │ │           │
└────┬────┘ └────┬─────┘ └───────┘└──────┘└───┬────┘ └────────┘ └───────────┘
     │           │                            │
     └─────┬─────┘                            │
           ▼                                  │
     ┌───────────────┐                        │
     │ blink-parser  │  manifest + lockfile    │
     └───────────────┘◄───────────────────────┘
                        (blink-report depends on blink-core + blink-analyzer)
```

## Crates

### `blink-parser`

The lowest layer: reads and parses manifest and lockfile *formats* —
`Cargo.toml`, `package.json`, `requirements.txt`, `Cargo.lock`,
`package-lock.json` — into plain structured data (`RawDependency`,
`LockedPackage`). It has no concept of "project," "framework," or
"health." Splitting this out means the file-format knowledge lives in one
place instead of being duplicated between detection and analysis.

### `blink-core`

The shared vocabulary every other crate builds on:

- `ProjectDetector` — uses `blink-parser` to read a directory's manifest,
  then interprets the result into a `Project`: language, framework,
  package manager, declared dependencies, file count, config file, and
  Rust-workspace/Python-virtualenv flags.
- `BlinkConfig` — reads and writes `blink.toml`, including the
  `[project].ignore` extra-ignore-directories list.
- `BlinkError` — the error type every fallible operation in `blink-core`
  returns.

Detection is intentionally manifest-based rather than heuristic-guessing
from file extensions: it's deterministic, fast, and doesn't require
installing dependencies first.

### `blink-analyzer`

Takes a `Project` (from `blink-core`) and a project root, and produces an
`AnalysisReport`:

- **Dependency graph & counts** — a shallow tree of direct dependencies
  from the manifest, plus a transitive count derived from the lockfile
  (via `blink-parser`).
- **Unused dependencies** — direct source-file scan for references to each
  *runtime* dependency. Dev dependencies are excluded.
- **Duplicate versions** — groups lockfile-resolved packages by name.
- **Large / largest dependencies** — measures installed size on disk
  (`node_modules` for JS/TS, the local Cargo registry cache for Rust).
- **Outdated dependencies** *(opt-in, `--online`)*.
- **Vulnerabilities** *(opt-in, `--online`, `blink security`)* — queries
  OSV.dev.
- **Health score** — a 0-100 heuristic derived from all of the above, plus
  `compute_health` for the three-way sub-score breakdown `blink health`
  shows (dependencies/configuration/code organization).
- **Recommendations** — `RecommendationEngine` groups findings into
  Performance/Maintenance/Security categories for `blink recommend`.

Every recommendation is derived directly from a finding — there's no
generic "enable compression"-style filler. See
[`docs/analysis.md`](analysis.md) for the exact rules and weights.

### `blink-report`

Formats an `AnalysisReport` for output: the health bar, the dependency
stats and largest-packages tables (`comfy-table`), a plain-text issue
list, the `JsonReport` shape, and full Markdown/HTML document renderers
for `blink report`. This crate does no terminal coloring or interactivity
of its own — everything it produces is plain text, a `serde`-serializable
struct, or a self-contained HTML string, so it stays simple to test and
reusable outside a terminal.

### `blink-cache`

Two distinct caches, both content-hash (SHA-256) based:

- A **project-local** build cache (`Cache`, persisted to
  `.blink/cache.json`) that `blink build` diffs a fresh scan against to
  report what actually changed.
- A **global**, per-user **analysis cache** (`AnalysisCache`, in the
  platform cache directory — `~/.cache/blink`, `~/Library/Caches/blink`,
  or `%LOCALAPPDATA%/blink`) that most analyzer-driven commands check
  before recomputing a result for an unchanged project. See
  [`docs/analysis.md`](analysis.md#the-global-analysis-cache).

### `blink-server`

- `FileWatcher` — wraps `notify`/`notify-debouncer-mini` with a 150ms
  debounce window and ignore rules for build output, dependency, and VCS
  directories, so a single save produces a single notification.
- `DevServer` — a small async static file server (`tokio` + raw HTTP/1.1
  parsing) used by `blink run`. It serves files from the project root and
  falls back to a placeholder page when no `index.html` exists yet.

### `blink-plugin`

Subprocess-based plugin discovery and execution, in the style of
`cargo`/`git` subcommands (see [`docs/plugins.md`](plugins.md) for the
full rationale and how to write one). No dynamic loading, no remote
registry — both deliberately, not as oversights.

### `blink-dashboard`

An interactive terminal UI (`ratatui`) for `blink dashboard`: a header,
health gauge, stats panel, and scrollable issues/suggestions list, backed
by the same `Project`/`AnalysisReport` data every other command uses.
Refreshes on a keypress or automatically via a background `FileWatcher`
thread. Rendering is tested headlessly with `ratatui`'s `TestBackend`
(no real terminal needed in CI).

### `blink-index`

The incremental project index that powers Blink's fast repeat commands. It
records every file's size, SHA-256 hash, modification time, language, line
count, and top-level symbols, persisted to `.blink/index.json`. A refresh
re-hashes and re-parses only the files whose size *or* mtime changed
(rayon-parallel) and reuses stored records for the rest, then answers
`search`/`symbols`/`hotspots`/`status`/`inspect` queries from memory.
Symbol extraction is a conservative, dependency-free line scanner covering
Rust/Python/TypeScript/JavaScript/Go — it prefers to miss an oddly
formatted declaration over inventing one, keeping with Blink's no-fabricated-
output rule. Depends only on `blink-core` (for the shared ignore list).

### `blink-workflow`

The fact-driven workflow engine behind the Phase 5–6 commands: rule-based
`optimize`, environment `doctor`, task discovery, `clean` planning, storage
`filesystem` analysis, `.env` comparison, duplicate-file detection, and
config auditing, plus a thin read-only `git` wrapper for `timeline`/
`hotspots`. Every finding names the concrete condition that produced it,
and nothing claims a speedup it did not measure. It reuses the heavy
per-file work (hashes, symbols) from `blink-index` and dependency findings
from `blink-analyzer` rather than recomputing them.

### `blink-cli`

Wires everything above into Blink's subcommands with a consistent
terminal UI (`colored` for styling, `indicatif` for spinners on
operations of unknown duration — skipped automatically on non-interactive
output via `console`'s TTY detection, so CI logs and piped output never
get spinner control characters mixed in). Only `blink run` and `blink
dashboard` need persistent runtimes (Tokio, and the terminal's raw
mode/alternate screen respectively); every other command is synchronous
and exits as soon as it's done.

### `packages/blink-cli` (npm)

Not a Rust crate — a small npm package (`bin/blink.js` shim +
`scripts/install.js` postinstall hook) that downloads the release binary
matching the user's platform/arch, verifies its SHA-256 checksum, and
puts a `blink` command on `PATH`. It contains no logic of Blink's own;
`bin/blink.js` only ever execs the downloaded native binary. See
`packages/blink-cli/README.md`.

## Design decisions worth knowing about

- **No fabricated numbers.** Every timing, size, and score shown in
  Blink's output is measured at run time or read from the filesystem, not
  hardcoded. The health score and its sub-scores are explicit, documented
  heuristics (see [`docs/analysis.md`](analysis.md)), never presented as
  more precise than they are. `blink benchmark` measures its own
  cache-hit numbers by actually warming and querying the real cache, not
  by asserting a plausible-looking figure.
- **Offline by default.** The only network calls anywhere in the codebase
  are the opt-in `--online` outdated-package check and `blink security`'s
  OSV.dev lookup.
- **Direct dependency graph, not a full resolver.** Building a complete
  transitive dependency tree accurately requires either a real package
  manager's resolution algorithm or a lockfile parser per ecosystem.
  Blink's graph is intentionally shallow (direct dependencies only);
  duplicate-version detection and transitive counts instead read the
  *already-resolved* lockfile, which is accurate without reimplementing
  resolution.
- **Format vs. domain logic stay separate.** `blink-parser` knows file
  formats; `blink-core`/`blink-analyzer` know what a "project" or a
  "duplicate dependency" means; `blink-report` knows how to present that.
  None of those concerns are duplicated across crate boundaries.
- **Extensibility without unsafe dynamic loading or a fake registry.**
  `blink-plugin`'s subprocess convention gives real extensibility (any
  language, no ABI to keep stable across Rust versions) without either
  the reliability cost of `dlopen`-style native plugin loading or
  pretending a package registry exists when it doesn't.
