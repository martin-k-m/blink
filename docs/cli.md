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

## `blink scan [path]`

Detects the project in `path` — language, framework, package manager,
file count, and dependency count — and prints a short report. Exits
non-zero if no recognizable manifest is found.

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

## Global

- `blink --help` / `blink <command> --help` — full flag reference for any
  command, generated by `clap` from the same definitions this document is
  written from.
- `blink --version` — print the installed version.
