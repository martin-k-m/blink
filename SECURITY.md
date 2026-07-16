# Security Policy

## Reporting a vulnerability

If you find a security vulnerability in Blink, please **do not** open a
public GitHub issue. Instead, use
[GitHub's private security advisory reporting](https://github.com/martin-k-m/blink/security/advisories/new)
for this repository. That creates a private discussion with the
maintainers where a fix can be developed before any public disclosure.

Please include:

- What the vulnerability is and its potential impact.
- Steps to reproduce it.
- Which Blink version(s) you've confirmed it in.

We'll acknowledge reports as promptly as we can and keep you updated as
the issue is investigated and (if confirmed) fixed.

## Scope

This covers the `blink` binary and every crate in this workspace
(`blink-core`, `blink-analyzer`, `blink-cache`, `blink-server`,
`blink-parser`, `blink-report`, `blink-plugin`, `blink-dashboard`,
`blink-cli`), as well as the npm distribution package
(`packages/blink-cli`).

A few things worth knowing if you're evaluating Blink's attack surface:

- **Network calls are opt-in and limited.** By default Blink makes no
  network requests at all. `--online` (outdated-package checks) and
  `blink security` (OSV.dev vulnerability lookups) are the only features
  that do, and both require the user to explicitly ask for them.
- **The plugin system execs local files, on purpose.** `blink <name>`
  running `blink-<name>` from `PATH` or `~/.blink/plugins` is the
  intended, documented behavior (see
  [`docs/plugins.md`](docs/plugins.md)) — it is not itself a
  vulnerability. A report that Blink "executes arbitrary code" via an
  installed plugin without more context (e.g. a way to get Blink to run
  a plugin the user didn't install) won't be actionable.
- **The npm installer verifies checksums.** `packages/blink-cli`'s
  postinstall script downloads a binary from a GitHub release and checks
  it against the release's published SHA-256 checksum before installing
  it.

If you're unsure whether something is in scope, report it anyway and
we'll sort it out.
