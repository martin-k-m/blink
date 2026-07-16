# @martin-k-m/blink

npm distribution for Blink, a Rust-powered developer context engine — the
context layer that gives developers and their tools reliable, local
understanding of any codebase.

```sh
npm install -g @martin-k-m/blink
blink --version
```

> Note: `blink-cli` (unscoped) is an unrelated, deprecated package — the
> scoped `@martin-k-m/blink` above is this tool.

This package contains no logic of its own. On install it downloads the
prebuilt `blink` binary matching your platform/architecture from the
matching GitHub release, verifies it against the release's published
SHA-256 checksum, and installs a thin shim on your `PATH` that execs it.

If your npm is configured to block install scripts, that download won't
run and `blink` won't be available — allow the postinstall for this
package, or [build from source](../../README.md#installation) instead.

## Supported platforms

- macOS: x64, arm64
- Linux: x64, arm64
- Windows: x64

These are the targets the release workflow is configured to build. On any
other platform (or today, since nothing is published), [build from
source](../../README.md#installation) instead.

## Full documentation

See the repository's `docs/` for the command reference, configuration, and
architecture — or `blink --help` once built.
