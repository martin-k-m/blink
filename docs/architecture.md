# Architecture

Blink is a Cargo workspace made up of five crates. Each one has a single
responsibility and no crate depends on a "sibling" it doesn't need — the CLI
is the only crate that wires everything together.

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
   └───────────────┘ └──────┬───────┘ └─────────────┘ └─────────────┘
                              │
                              └── depends on blink-core's Project type
```

## Crates

### `blink-core`

The shared vocabulary every other crate builds on:

- `ProjectDetector` — inspects a directory's manifest files
  (`Cargo.toml`, `package.json`, `requirements.txt` / `pyproject.toml`) and
  produces a `Project`: language, framework, package manager, declared
  dependencies, and a file count.
- `BlinkConfig` — reads and writes `blink.toml`.
- `BlinkError` — the error type every fallible operation in `blink-core`
  returns.

Detection is intentionally manifest-based rather than heuristic-guessing
from file extensions: it's deterministic, fast, and doesn't require
installing dependencies first.

### `blink-analyzer`

Takes a `Project` (from `blink-core`) and a project root, and produces an
`AnalysisReport`:

- **Dependency graph** — a shallow tree of direct dependencies, built from
  the manifest itself (not a full transitive resolution).
- **Unused dependencies** — direct source-file scan for import/`use`
  references to each *runtime* dependency. Dev dependencies are excluded
  from this check since tools like `typescript` or `eslint` are typically
  invoked from config files rather than imported.
- **Duplicate versions** — parses `Cargo.lock` / `package-lock.json` to
  find packages resolved at more than one version.
- **Large dependencies** — measures installed size under `node_modules`
  (only meaningful once dependencies are actually installed).
- **Outdated dependencies** *(opt-in, `--online`)* — queries crates.io /
  the npm registry for each dependency's latest published version. This is
  the only part of Blink that touches the network, and it's off by
  default.

Every recommendation `blink analyze` prints is derived directly from one of
these findings — there's no generic "enable compression"-style filler.

### `blink-cache`

A content-hash (SHA-256) fingerprint of every tracked file in a project,
persisted to `.blink/cache.json`. `blink build` diffs a fresh scan against
the previous snapshot to report what actually changed, so unchanged files
are never treated as new work.

### `blink-server`

- `FileWatcher` — wraps `notify`/`notify-debouncer-mini` with a 150ms
  debounce window and ignore rules for build output, dependency, and VCS
  directories, so a single save produces a single notification.
- `DevServer` — a small async static file server (`tokio` + raw HTTP/1.1
  parsing) used by `blink run`. It serves files from the project root and
  falls back to a placeholder page when no `index.html` exists yet.

### `blink-cli`

Wires the crates above into five subcommands (`init`, `scan`, `analyze`,
`run`, `build`) with a consistent terminal UI. Only `blink run` spins up a
Tokio runtime — the other commands are synchronous, so they start and exit
immediately.

## Design decisions worth knowing about

- **No fabricated numbers.** Every timing and size shown in Blink's output
  (`Ready in: 42ms`, `Scan completed: 2ms`, cache file counts, bundle
  sizes) is measured at run time with `std::time::Instant` or read from the
  filesystem, not hardcoded.
- **Offline by default.** The only network call anywhere in the codebase is
  the opt-in `--online` outdated-package check in `blink analyze`.
- **Direct dependency graph, not a full resolver.** Building a complete
  transitive dependency tree accurately requires either a real package
  manager's resolution algorithm or a lockfile parser per ecosystem.
  Blink's v0.1 graph is intentionally shallow (direct dependencies only);
  duplicate-version detection instead reads the *already-resolved*
  lockfile, which is accurate without reimplementing resolution.
