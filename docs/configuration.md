# Configuration

`blink init` writes a `blink.toml` file to the project root:

```toml
[project]
name = "my-app"

[server]
port = 3000

[optimization]
cache = true
analyze = true
```

| Table            | Key       | Type      | Default | Description                                              |
| ---------------- | --------- | --------- | ------- | ---------------------------------------------------------|
| `[project]`       | `name`    | `string`  | —       | Project name. Seeded from the detected manifest.         |
| `[server]`        | `port`    | `integer` | `3000`  | Default port for `blink run`. Overridden by `--port`.    |
| `[optimization]`  | `cache`   | `bool`    | `true`  | When `false`, `blink build` scans files but skips reading/writing `.blink/cache.json`. |
| `[optimization]`  | `analyze` | `bool`    | `true`  | Reserved for running analysis automatically during `blink build` (see [roadmap](roadmap.md)). |

`blink.toml` is optional. Every command falls back to sensible defaults
(port `3000`, no cache) if it's missing, and `blink init` is idempotent —
it's safe to re-run.
