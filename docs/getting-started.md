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

## Your first look

From inside any Rust, TypeScript/JavaScript, or Python project:

```sh
cd my-project
blink inspect
```

`inspect` is the one command to run first: it tells you what the project
is, how to run it, where its entry points are, and what tasks it defines —
one screen, no configuration required, nothing installed or executed
beyond reading files that are already there. (`blink scan` is the smaller,
detection-only version if that's all you want.)

Two more first-day commands:

```sh
blink doctor   # can my environment actually build this? (runtimes, tools)
blink setup    # install dependencies and prepare a fresh clone (asks first)
```

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

## Find your way around

```sh
blink optimize      # a scored, rule-based list of concrete improvements
blink search Config # instant indexed search across the codebase
blink symbols       # every function/type/class Blink found
blink tasks         # the project's own commands, ready to run
blink task test     # run one of them (mirrors its exit code)
```

## Keep it running

```sh
blink watch     # re-analyze on every file change
blink run        # a local dev server + the same watching
blink dashboard  # an interactive terminal view of the above
```

## Configure it (optional)

Drop a `blink.toml` — or `.bnk`, the same schema under a signature name —
in your project root to name tasks, tune ignores, or define profiles.
Validate it any time with `blink config check`. See
[`docs/configuration.md`](configuration.md).

## Where to go next

- [`docs/cli.md`](cli.md) — every command and flag.
- [`docs/FEATURE_AUDIT.md`](FEATURE_AUDIT.md) — a per-command inventory.
- [`docs/analysis.md`](analysis.md) — exactly what each score measures,
  including where it's a documented heuristic rather than a precise
  number.
- [`docs/configuration.md`](configuration.md) — `blink.toml`/`.bnk` reference.
- [`docs/plugins.md`](plugins.md) — extending Blink with your own
  subcommands.
- [`docs/architecture.md`](architecture.md) — how the crates fit together,
  for contributors.
