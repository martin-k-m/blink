# CLI Reference

```
blink <COMMAND>
```

Every command accepts an optional `path` argument (defaults to `.`).

## `blink init [path]`

Creates a `blink.toml` in `path`, creating the directory first if it
doesn't exist. If `path` already contains a recognizable project, the
config is seeded with that project's detected name; otherwise it falls
back to the directory name.

## `blink scan [path] [--verbose]`

Detects the project in `path` — language, framework, package manager,
file count, and dependency count — and prints a short report. Exits
non-zero if no recognizable manifest is found.

| Flag              | Effect                                                        |
| ----------------- | ------------------------------------------------------------- |
| `-v`, `--verbose` | Print the diagnostic view instead: the resolved path, which manifest file was matched, whether it's a workspace, and the full list of ignore rules in effect (Blink's built-ins plus anything `[project].ignore`/`[scan].ignore` adds). Useful when a file count looks wrong and you need to see what was skipped. |

## `blink analyze [path] [--online] [--json]`

Runs Blink's dependency analyzer and prints a report: dependency counts,
a health score, the largest installed dependencies, any detected issues
(unused/duplicate/large/outdated dependencies), and suggestions derived
directly from those findings. See [`docs/analysis.md`](analysis.md) for
exactly what each part measures.

| Flag       | Effect                                                                 |
| ---------- | ------------------------------------------------------------------------|
| `--online` | Also check crates.io / the npm registry for outdated packages. Requires network access. |
| `--json`   | Print the report as JSON instead of formatted terminal output. Suitable for piping into other tools. |

## `blink run [path] [--port <PORT>]`

Starts a local dev server (defaulting to the port in `blink.toml`, or
`3000`) and a debounced file watcher. Saving a file triggers a dependency
graph rebuild and cache invalidation for the changed files, printed to the
terminal. Stop with Ctrl+C.

## `blink build [path]`

Runs a cache-aware file scan: hashes every tracked file and compares
against the snapshot from the previous `blink build`, reporting how many
files were unchanged/changed/added/removed. Set `[optimization].cache =
false` in `blink.toml` to skip the cache entirely (files are still
scanned and counted, but nothing is read from or written to
`.blink/cache.json`).

Note: as of this release, `blink build` is cache bookkeeping, not a
bundler — see [`docs/roadmap.md`](roadmap.md) for what that means and
what's planned.

## `blink deps [path]`

A focused view of dependency counts, the largest installed packages, and
detected issues — the parts of `blink analyze` about dependencies
specifically, without the health score or performance section.

## `blink health [path] [--json]`

Prints a project health score broken into three independently measurable
sub-scores (dependencies, configuration, code organization) plus
suggestions for each. See [`docs/analysis.md`](analysis.md) for the exact
rubric — nothing here is an invented number.

## `blink recommend [path] [--online]`

Prints rule-based recommendations grouped into three categories
(Performance, Maintenance, Security). Each rule checks one concrete fact
(a config value, a dependency finding, a vulnerability lookup) — see
[`docs/analysis.md`](analysis.md). `--online` additionally runs the
outdated-package and vulnerability checks; without it, those two rules
report "unknown" rather than guessing.

## `blink watch [path]`

Like `blink run`'s file watcher, but without the dev server: re-runs
analysis on every file change and prints what changed. Useful when you
just want live dependency feedback while editing, not a local server.

## `blink ci [path] [--online]`

Runs analysis and exits with a status code meant for pipelines:

| Exit code | Meaning                                          |
| --------- | ------------------------------------------------- |
| `0`       | Pass — no warnings.                                |
| `1`       | Pass with warnings (non-security issues found).    |
| `2`       | Failure — the project couldn't be analyzed, or (with `--online`) a known vulnerability was found. |

This is the one command in Blink with non-standard exit codes; every other
command uses the usual `0` success / `1` error convention.

## `blink security [path] [--json]`

Checks every declared dependency against [OSV.dev](https://osv.dev)
(Google's free, open Open Source Vulnerabilities database) for known
vulnerabilities. Requires network access — this always makes one, unlike
`--online` elsewhere, since checking vulnerabilities is this command's
entire purpose. Reports vulnerability IDs (e.g. `GHSA-...`, `RUSTSEC-...`)
without fetching or summarizing the underlying advisories; look those up
directly if you need details.

## `blink report [path] [--json|--markdown|--html] [--output <file>] [--online]`

Exports a full project report. Defaults to JSON on stdout; `--markdown`
and `--html` produce a complete document (not a fragment) suitable for a
PR description, a wiki page, or archiving. `--output <file>` writes to a
file instead of stdout.

## `blink plugins [install <path> --name <name>]`

Lists discovered plugins (executables named `blink-<name>` on `PATH` or in
`~/.blink/plugins`), or installs one by copying a local executable into
`~/.blink/plugins`. There is no remote plugin registry — `install` only
ever copies a file you already have. See
[`docs/plugins.md`](plugins.md) for how the plugin convention works and
how to write one.

Any command Blink doesn't recognize is checked against installed plugins
before failing: `blink foo` runs `blink-foo` if it's installed, forwarding
all arguments and the exit code.

## `blink benchmark [path] [--runs <N>]`

Measures Blink's own performance against `path`: process startup time (the
minimum of `--runs` fresh launches, default 3), project scan time, a cold
analysis run, and a cache-hit analysis lookup. Every number is measured on
that run — see [`docs/analysis.md`](analysis.md) for how "cached" is
measured without just running the analysis twice.

## `blink dashboard [path]`

Opens an interactive terminal dashboard (built on `ratatui`) showing the
project's health, stats, and issues, refreshing automatically when a
watched file changes. Keys: `r` refresh, `o` toggle online checks, `q` or
`Esc` quit.

## Context engine

Blink's context layer builds a single, serializable **context graph** of
the project — its identity, stats, areas, dependencies, commands, files and
symbols, and the file→file references between them — and answers questions
against that model. It's built on the same incremental index as the
commands below; everything it shows is measured or resolved from real
project files, never invented. References are resolved conservatively
(TS/JS relative imports, Python imports, Rust `mod` declarations, and
cross-crate `<crate>::` references inside a Cargo workspace); an import
that can't be matched to a real project file is never turned into an edge.
See [`docs/context-engine.md`](context-engine.md) for the full model.

### `blink context [path] [--json]`

Builds the context graph and prints an understanding report: project
identity, stats, the most important areas (ranked by symbol count), and the
project's commands. `--json` emits the full graph.

### `blink query <query> [path] [--limit <N>] [--json]`

Deterministic, local structured search over the context graph. The query
is tokenized (camelCase split, stop/question words like "where"/"is"/"the"
dropped) and areas, files, symbols, dependencies, and commands are ranked
by name match, returned grouped. This is lexical search over the local
model — not AI, no inference. `--limit` caps results per group (default
`10`); `--json` emits structured results.

### `blink explain <file> [path] [--json]`

Explains one project-relative file using only real signals: the file's own
leading doc comment (verbatim), the top-level symbols it declares, the
project files it imports, the external packages it imports, and the project
files that import it. Nothing is inferred — there are no invented
"responsibilities."

### `blink map [path] [--format terminal|markdown|json|graph]`

An architecture view of the project: areas ranked by symbol count and the
area→area dependency edges derived from resolved references. `--format`
selects `terminal` (default), `markdown`, `json`, or `graph` (a Mermaid
architecture graph).

### `blink export [path] [--format json|yaml|markdown|graph] [--output <file>]`

Exports the full context graph. `--format` selects `json` (default),
`yaml`, `markdown`, or `graph` (Mermaid). Writes to stdout by default;
`--output <file>` writes a file instead. Conventional filenames are
`blink-context.json` / `.yaml` / `.md` / `.mmd`.

## Project intelligence

These commands build on a fast, incremental on-disk index
(`.blink/index.json`) — see [`docs/analysis.md`](analysis.md#the-project-index).
Report commands support `--json`.

### `blink inspect [path] [--json]`

A one-screen answer to "I just cloned this — what is it and how do I run
it?": language/framework/package manager, measured file/line/symbol
counts and size, the natural run command, conventional entry-point files,
discovered tasks, and a Git snapshot. Everything shown is measured or
detected, never guessed.

### `blink optimize [path] [--json]`

A rule-based optimization report with a 0–100 score and concrete
suggestions. Each category (Dependencies, Project Structure, Duplicate
Files, Tests, Documentation, Configuration) is `✓`/`⚠` based on a specific
measured condition. The score deducts a fixed amount per warning; the
per-category findings are the substance. See
[`docs/analysis.md`](analysis.md#the-optimize-score). No build-speed
improvement is ever claimed, because none is measured here.

### `blink duplicates [path] [--json]`

Finds files with byte-for-byte identical contents (matched by SHA-256 via
the index) and reports how much space consolidating each group would
reclaim. Empty files are ignored.

### `blink doctor [path] [--json]`

Checks that this project can actually be developed here: required runtimes
and package managers on `PATH`, Git, Docker (only if a compose file is
present), and any environment-variable **names** declared in
`.env.example` but missing from `.env`. Never prints a variable's value.
`✓` present · `⚠` optional missing · `❌` required missing.

### `blink filesystem [path] [--json]`

Shows where the repository's bytes live: total size, how much is
regenerable (build output, dependencies, caches) versus source, and a
per-top-level-entry breakdown.

### `blink config-audit [path] [--json]`

Checks which standard project-config files are present (README, LICENSE,
CONTRIBUTING, `.gitignore`, `.editorconfig`, CI configuration). Presence
only — it doesn't judge contents.

### `blink docs [path] [--output <file>]`

Generates a Markdown project summary from measured facts (overview,
statistics, language breakdown, discovered tasks), with the generated
section clearly marked for review. Writes to stdout, or to `--output`.

## Indexing

### `blink index [path] [--rebuild]`

Builds or incrementally refreshes the index. A refresh re-hashes and
re-parses only files whose size or modification time changed and reuses
stored records for the rest; `--rebuild` forces a full rebuild.

### `blink status [path] [--json]`

Reports index state: file/symbol/line counts, on-disk size, and a
per-language breakdown — or that the project isn't indexed yet.

### `blink search <query> [path] [--symbols] [--json]`

Searches indexed file paths for `query` (case-insensitive), or symbol
names with `--symbols`.

### `blink symbols [path] [--filter <text>] [--json]`

Lists discovered top-level symbols (functions, structs, enums, traits,
classes, interfaces, type aliases) across Rust, Python, TypeScript,
JavaScript, and Go, optionally filtered by name.

### `blink hotspots [path] [--limit <N>] [--json]`

Shows the largest files (from the index) and the most-frequently-changed
files (from local Git history) — a maintenance-risk view.

### `blink timeline [path] [--limit <N>]`

Recent development activity from local Git history: commit count, recently
changed files, and recent commit subjects.

## Daily workflow

### `blink tasks [path] [--json]`

Lists tasks discovered from `[commands]` in blink.toml/.bnk, `package.json`
scripts, `Makefile` targets, `justfile` recipes, and Cargo aliases —
showing the underlying command and its source so nothing is hidden.

### `blink task <name> [path] [--dry-run]`

Runs a discovered task by name, mirroring its exit code (so `blink task
test` works in CI). `[commands]` entries take precedence over same-named
tasks from other sources. `--dry-run` prints the command without running it.

### `blink profile <name> [path] [--dry-run]`

Runs a `[profiles.<name>]` command sequence in order, stopping on the
first non-zero exit.

### `blink clean [path] [--dry-run] [--all] [--yes]`

Removes regenerable cache/build directories, showing sizes first and
asking before deleting. By default it leaves "heavy" artifacts (`target`,
`node_modules`, virtualenvs) that cost a reinstall/recompile — `--all`
includes them. `--dry-run` shows the plan without deleting; `--yes` skips
the prompt.

### `blink env [path] [--json]`

Compares `.env` against `.env.example`, reporting configured / missing /
unused variable **names** only — never values.

### `blink check [path]`

Runs the project's real local checks and exits non-zero if any fail: for
Rust, `cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test`;
for Node, the project's own lint/test scripts; for Python, `ruff`/`pytest`
when available. Complements `blink ci` (which is about analysis exit
codes, not running your toolchain).

### `blink setup [path] [--yes]`

Prepares a freshly cloned project: copies `.env.example` to `.env` when
missing and installs dependencies with the detected package manager. Shows
the plan and asks first unless `--yes` is given.

### `blink completions <shell>`

Prints a shell completion script for `bash`, `zsh`, `fish`, `powershell`,
or `elvish` to stdout. Redirect it into the location your shell loads
completions from.

### `blink config check [path]`

Validates the project's `blink.toml`/`.bnk` and reports issues (e.g. a
profile with no commands). See [`docs/configuration.md`](configuration.md).

## Global

- `blink --help` / `blink <command> --help` — full flag reference for any
  command, generated by `clap` from the same definitions this document is
  written from.
- `blink --version` — print the installed version.
