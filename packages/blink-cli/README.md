# blink-cli

npm distribution for [Blink](https://github.com/martin-k-m/blink), a
Rust-powered developer acceleration toolkit.

```sh
npm install -g blink-cli
blink --version
blink scan
```

This package contains no logic of its own. `npm install` downloads the
prebuilt `blink` binary matching your platform/architecture from a
[GitHub release](https://github.com/martin-k-m/blink/releases), verifies
it against the release's published SHA-256 checksum, and installs a thin
shim on your `PATH` that execs it.

## Supported platforms

- macOS: x64, arm64
- Linux: x64, arm64
- Windows: x64

If your platform isn't listed, install isn't supported yet — [build from
source](https://github.com/martin-k-m/blink#installation) instead.

## Full documentation

See the [main repository](https://github.com/martin-k-m/blink) for command
reference, configuration, and architecture docs.
