# Architecture

Blink is a Cargo workspace of fourteen crates, plus an npm package that
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
`package-lock.json`, `yarn.lock`, `pnpm-lock.yaml` — into plain structured
data (`RawDependency`, `LockedPackage`). Cargo and npm locks also yield
dependency *edges* and a direct-dependency set, which is what lets the
security audit show which declared dependency pulls a vulnerable package
in; yarn and pnpm yield resolved versions only. It has no concept of
"project," "framework," or
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
  OSV.dev for every package the *lockfile* resolved, not just the declared
  manifest entries, and classifies each finding as direct or transitive
  with the dependency path that pulls it in. An unreachable advisory
  source is reported as such, never as "clean".
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

### `blink-context`

Builds the project **context graph** — the shared model the context engine
answers questions against. It unifies three existing sources into one
serializable `ContextGraph`: `blink-core`'s detection (project identity),
`blink-index`'s files and symbols, and the manifest's declared
dependencies, plus the project's commands and the file→file references
between its files. References are resolved *conservatively*: TS/JS relative
imports, Python absolute/relative imports, Rust `mod` declarations, and —
inside a Cargo workspace — Rust `<crate>::` references (in a `use` or an
inline path) resolved to that crate's `src/lib.rs`/`main.rs`. An import
that can't be resolved to a real project file is **never** turned into an
invented edge. Files are grouped into "areas" — a directory grouping where
top-level files are `(root)` and files under a container dir (`src`,
`crates`, `packages`) group by container plus next segment (`src/auth`,
`crates/blink-core`). Depends on `blink-core`, `blink-index`,
`blink-workflow`, `serde`, `rayon`, and `toml`.

### `blink-query`

Deterministic, **local** structured search over the `ContextGraph`. It
tokenizes the query (splitting camelCase), drops stop and question words
("where", "is", "the", ...), and ranks areas, files, symbols, dependencies,
and commands by name match. It is lexical search over the local model — not
AI, and it performs no inference. Kept separate from `blink-context` so the
graph-building and graph-querying concerns don't bleed into each other.
Depends only on `blink-context`.

### `blink-export`

Serializes a `ContextGraph` to JSON, YAML, Markdown, and a Mermaid
architecture graph. The YAML path uses a small internal block-style emitter
rather than pulling in an external YAML dependency. Separating export from
both graph-building and querying keeps output-format knowledge in one place
(mirroring how `blink-report` isolates analysis formatting). Depends on
`blink-context` and `serde_json`.

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
- **The context engine invents no edges.** The `ContextGraph`
  (`blink-context`) is built entirely from measured facts — detection,
  the index's files/symbols, declared dependencies — and its file→file
  references are resolved *conservatively*: an import that doesn't map to
  a real project file is dropped, never guessed into an edge. `blink-query`
  is deliberately deterministic lexical search over that model with no
  inference, and `blink-export` only serializes what the graph already
  contains. Building the graph, searching it, and serializing it are three
  separable concerns kept in three crates over one shared model, so none
  of them grows knowledge of the others' job.
