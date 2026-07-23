# Configuration

`blink init` writes a `blink.toml` file to the project root:

```toml
[project]
name = "my-app"
type = "web"
ignore = ["vendor", "generated"]

[server]
port = 3000

[optimization]
cache = true
analyze = true

# Optional: named commands surfaced by `blink tasks` and run with `blink task`.
[commands]
dev = "npm run dev"
build = "npm run build"
test = "npm test"

# Optional: on-disk index behavior (see docs/analysis.md).
[index]
enabled = true
auto_update = true

# Optional: the context engine (blink context/query/map/explain/export).
[context]
enabled = true
include = ["src", "crates"]  # empty = whole project

# Optional: preferred `blink report` format when none is given on the CLI.
[report]
format = "markdown"

# Optional: named command sequences run with `blink profile <name>`.
[profiles.dev]
commands = ["docker compose up -d", "npm install"]

# Optional: free-form config for plugins. Blink core never interprets these.
[plugins.react]
component_paths = ["src/components"]
```

## `.bnk` — the same file, a shorter name

`.bnk` is accepted **as an alternate filename for the exact same schema**.
A project may use either `blink.toml` or `.bnk`; the parser, defaults, and
every command behave identically. If both are present, `.bnk` wins. The
point is recognizability — seeing a `.bnk` in a repository signals "this
project uses Blink tooling" at a glance — without introducing a second,
divergent config format to keep in sync. `blink config check` validates
whichever file is present.

| Table            | Key           | Type              | Default | Description                                              |
| ---------------- | ------------- | ----------------- | ------- | ---------------------------------------------------------|
| `[project]`       | `name`        | `string`          | —       | Project name. Seeded from the detected manifest.         |
| `[project]`       | `type`        | `string`          | —       | Optional descriptive label (`"web"`, `"cli"`, ...), shown by `blink inspect`. Purely descriptive. |
| `[project]`       | `ignore`      | `array<string>`   | `[]`    | Extra directory names to skip during scans, in addition to Blink's built-in list (`.git`, `node_modules`, `target`, `dist`, `build`, `.next`, `.turbo`, `.cache`, `.blink`, `coverage`, `__pycache__`, `.venv`, `venv`). Merged with `[scan].ignore`. Run `blink scan --verbose` to print the list actually in effect. |
| `[scan]`          | `ignore`      | `array<string>`   | `[]`    | Alias/companion of `[project].ignore` (the two are merged and de-duplicated). Exists because the Phase 8 config shape uses `[scan]`. |
| `[server]`        | `port`        | `integer`         | `3000`  | Default port for `blink run`. Overridden by `--port`.    |
| `[optimization]`  | `cache`       | `bool`            | `true`  | When `false`, `blink build` scans files but skips reading/writing `.blink/cache.json`; `blink recommend`/`ci` also flag caching as disabled. |
| `[optimization]`  | `analyze`     | `bool`            | `true`  | Reserved for running analysis automatically during `blink build` (see [roadmap](roadmap.md)). |
| `[commands]`      | *(any)*       | `string`          | `{}`    | Named tasks. `blink tasks` lists them; `blink task <name>` runs one. Override same-named tasks discovered elsewhere (package.json scripts, Makefile targets, ...). |
| `[index]`         | `enabled`     | `bool`            | `true`  | When `false`, index-backed commands build a throwaway in-memory index instead of persisting `.blink/index.json`. |
| `[index]`         | `auto_update` | `bool`            | `true`  | When `true`, index-backed commands incrementally refresh and save the index before running. |
| `[context]`       | `enabled`     | `bool`            | `true`  | When `false`, the context-engine commands (`context`/`query`/`map`/`explain`/`export`) refuse to run. |
| `[context]`       | `include`     | `array<string>`   | `[]`    | Project-relative path roots the context graph is limited to. Empty (the default) covers the whole project. A root matches on a path-segment boundary — `"src"` covers `src/main.rs` but not `srcgen/x.rs`. |
| `[report]`        | `format`      | `string`          | —       | Preferred `blink report` format (`"json"`/`"markdown"`/`"html"`) when none is passed on the CLI. |
| `[profiles.<name>]` | `commands`  | `array<string>`   | `[]`    | An ordered command sequence run by `blink profile <name>`; stops on the first non-zero exit. |
| `[plugins.<name>]` | *(any)*      | *(any TOML)*      | `{}`    | Opaque per-plugin configuration. Preserved and exposed to plugins; Blink core never reads it. |

`blink.toml`/`.bnk` is optional. Every command falls back to sensible
defaults if it's missing, and `blink init` is idempotent — it's safe to
re-run. `blink config check` validates the file and surfaces a `Context`
line reporting whether the context engine is enabled and which `include`
roots (if any) it's limited to.

## Two different caches

`[optimization].cache` controls the **project-local** build cache
(`.blink/cache.json`, used by `blink build`). It doesn't affect Blink's
separate **global** per-user analysis cache (in your platform's cache
directory), which most analysis-driven commands use automatically and
which isn't currently configurable — see
[`docs/analysis.md`](analysis.md#the-global-analysis-cache) for how that
one works and why `--online` results are never cached.
