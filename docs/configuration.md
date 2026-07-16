# Configuration

`blink init` writes a `blink.toml` file to the project root:

```toml
[project]
name = "my-app"
ignore = ["vendor", "generated"]

[server]
port = 3000

[optimization]
cache = true
analyze = true
```

| Table            | Key       | Type            | Default | Description                                              |
| ---------------- | --------- | --------------- | ------- | ---------------------------------------------------------|
| `[project]`       | `name`    | `string`        | —       | Project name. Seeded from the detected manifest.         |
| `[project]`       | `ignore`  | `array<string>` | `[]`    | Extra directory names to skip during scans, in addition to Blink's built-in list (`.git`, `node_modules`, `target`, `dist`, `build`, `.next`, `.turbo`, `.cache`, `.blink`, `__pycache__`, `.venv`, `venv`). Applies to project detection (`blink scan`'s file count) and `blink build`'s cache. |
| `[server]`        | `port`    | `integer`       | `3000`  | Default port for `blink run`. Overridden by `--port`.    |
| `[optimization]`  | `cache`   | `bool`          | `true`  | When `false`, `blink build` scans files but skips reading/writing `.blink/cache.json`; `blink recommend`/`ci` also flag caching as disabled. |
| `[optimization]`  | `analyze` | `bool`          | `true`  | Reserved for running analysis automatically during `blink build` (see [roadmap](roadmap.md)). |

`blink.toml` is optional. Every command falls back to sensible defaults
(port `3000`, no cache) if it's missing, and `blink init` is idempotent —
it's safe to re-run.

## Two different caches

`[optimization].cache` controls the **project-local** build cache
(`.blink/cache.json`, used by `blink build`). It doesn't affect Blink's
separate **global** per-user analysis cache (in your platform's cache
directory), which most analysis-driven commands use automatically and
which isn't currently configurable — see
[`docs/analysis.md`](analysis.md#the-global-analysis-cache) for how that
one works and why `--online` results are never cached.
