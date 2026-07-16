# blink-cli (npm distribution — pre-release, not published)

> **This package is not published.** Blink is pre-release: there is no npm
> release and no GitHub release for it to download. **Do not** run
> `npm install -g blink-cli` expecting this tool — that name currently
> belongs to an unrelated, deprecated package. Install Blink by
> [building from source](../../README.md#installation) instead. A real,
> reserved package name is future work.

npm distribution for Blink, a Rust-powered developer acceleration toolkit.

Once published (under a finalized name), installing will look like this,
downloading the prebuilt `blink` binary and putting it on your `PATH`:

```sh
# planned — not yet available
npm install -g <published-name>
blink --version
```

This package contains no logic of its own. On install it downloads the
prebuilt `blink` binary matching your platform/architecture from the
matching GitHub release, verifies it against the release's published
SHA-256 checksum, and installs a thin shim on your `PATH` that execs it.

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
