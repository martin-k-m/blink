# Contributing to Blink

Thanks for considering a contribution. This project aims to stay a
genuinely useful, honestly-documented developer tool — see
[`docs/architecture.md`](docs/architecture.md)'s "Design decisions worth
knowing about" section for the standards that guides code review here:
no fabricated numbers, no fake output, heuristics documented as
heuristics.

## Getting set up

You need a stable Rust toolchain (1.75+) and Cargo. For the npm package,
Node.js 16+.

```sh
git clone https://github.com/martin-k-m/blink.git
cd blink
cargo build --workspace
cargo test --workspace
```

## Making a change

1. Fork the repo and create a branch off `main`.
2. Make your change. If it's user-facing, update the relevant doc in
   `docs/` and/or `README.md` in the same PR — undocumented behavior
   isn't considered done here.
3. Add or update tests. See "Testing expectations" below for what that
   means for different kinds of changes.
4. Run the full check suite locally before opening a PR:

   ```sh
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```

   These are exactly what CI runs — there's no separate, stricter CI-only
   check waiting to surprise you.
5. Open a pull request describing what changed and why. Link any related
   issue.

## Testing expectations

- **New analyzer behavior** (a new check, a new field on a report):
  a unit test in the crate that owns the logic, and — if it's
  user-visible — an integration test in `tests/cli_tests.rs` or a new
  fixture under `tests/fixtures/`.
- **New CLI commands/flags:** at least one integration test exercising
  the happy path via `assert_cmd`.
- **Network-dependent behavior** (`--online`, `blink security`): follow
  the existing pattern — cover the *offline* path automatically (assert
  the feature is skipped/reported as "unknown" without the flag), and
  verify the online path manually rather than adding a flaky
  network-dependent test to CI.
- **TUI changes** (`blink-dashboard`): a headless test using `ratatui`'s
  `TestBackend` — see `crates/blink-dashboard/src/tests.rs` for the
  pattern.

## Code style

`cargo fmt` and `clippy -D warnings` are the source of truth; don't
hand-wave style debates the tools already settle. Beyond that:

- Prefer editing existing files over creating new ones.
- Don't add abstractions, config flags, or "just in case" flexibility
  beyond what the current change needs.
- Comments explain *why*, not *what* — well-named code should make the
  "what" obvious on its own.
- If a number or claim in output/docs isn't measured or derived from a
  concrete, checkable fact, don't ship it. See
  [`docs/analysis.md`](docs/analysis.md) for the standard this project
  holds itself to on that point.

## Reporting bugs / requesting features

Use [GitHub Issues](https://github.com/martin-k-m/blink/issues). For
security vulnerabilities, see [`SECURITY.md`](SECURITY.md) instead —
please don't file those as public issues.

## Code of Conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md).
