# Getting Started

## Install

```sh
npm install -g blink-cli
```

This downloads the `blink` binary for your platform from a
[GitHub release](https://github.com/martin-k-m/blink/releases) and puts a
`blink` command on your `PATH`. Supported: macOS (x64, arm64), Linux (x64,
arm64), Windows (x64). If your platform isn't in that list, or you'd
rather not use npm, [build from source](../README.md#installation)
instead — the npm package is a distribution mechanism, not a dependency
of the tool itself.

Verify it worked:

```sh
blink --version
```

## Your first scan

From inside any Rust, TypeScript/JavaScript, or Python project:

```sh
cd my-project
blink scan
```

Blink reads your project's manifest (`Cargo.toml`, `package.json`,
`requirements.txt`/`pyproject.toml`) and reports what it found — no
configuration required, nothing installed or executed beyond reading
files that are already there.

## Understand dependency health

```sh
blink analyze
```

Prints a health score, dependency counts, the largest installed packages,
and any issues (unused dependencies, duplicate versions, oversized
packages) with recommendations derived directly from what was found. Add
`--online` to also check for outdated packages and known vulnerabilities
(the only commands that touch the network — everything else is fully
offline). Add `--json` to get the same data as JSON for scripting:

```sh
blink analyze --json | jq '.health.score'
```

## Keep it running

```sh
blink watch     # re-analyze on every file change
blink run        # a local dev server + the same watching
blink dashboard  # an interactive terminal view of the above
```

## Where to go next

- [`docs/cli.md`](cli.md) — every command and flag.
- [`docs/analysis.md`](analysis.md) — exactly what the analyzer measures,
  including where it's a documented heuristic rather than a precise
  number.
- [`docs/configuration.md`](configuration.md) — `blink.toml` reference.
- [`docs/plugins.md`](plugins.md) — extending Blink with your own
  subcommands.
- [`docs/architecture.md`](architecture.md) — how the crates fit together,
  for contributors.
