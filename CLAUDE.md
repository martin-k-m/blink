# CLAUDE.md

Context for Claude Code (or any agent) picking up this repository. Read
this before making changes — it captures decisions and environment
quirks that aren't obvious from the code alone.

## What this project is

Blink is a Rust-powered developer **context engine**: it builds reliable,
local, deterministic understanding of any codebase — a *context graph* of
its identity, areas, dependencies, files, symbols, and references — for
humans and their tools. Around that core it also does project detection,
dependency health analysis, a dev server, a build cache, a plugin system,
and an interactive terminal dashboard, distributed via npm. (The tagline is
"The context layer for modern software development." AI can generate code
but struggles to *understand* existing codebases; Blink is the missing
context layer — local-first, offline by default, no fabricated output, no
LLM required. The "workflow engine" — `optimize`/`doctor`/`tasks`/`clean` —
remains one feature among many, not the product's category.) It was built
in this repo from an empty skeleton (`README.md` + `LICENSE`) up through
several shipped milestones (v0.1–v0.6). See `docs/roadmap.md` for what
shipped in each and `docs/architecture.md` for how the fourteen crates fit
together.

**Current status:** v0.6 shipped the **context engine** on top of v0.5 —
three new crates (`blink-context` builds the serializable context graph,
`blink-query` runs deterministic local search over it, `blink-export`
serializes it to JSON/YAML/Markdown/Mermaid) and 5 new subcommands
(`context`/`query`/`map`/`explain`/`export`), plus a `[context]` config
table (`enabled`/`include`) and cross-file + cross-crate reference
resolution (TS/JS/Python imports, Rust `mod`, and workspace `<crate>::`
references — an import that can't be matched to a real project file is never
turned into an edge). This built on v0.5, which shipped the "phase 5–8" work
on top of v0.4 — a new incremental index (`blink-index`), a fact-driven
workflow engine (`blink-workflow`), and 21 subcommands (`inspect`/`optimize`/
`doctor`/`index`/`search`/`symbols`/`hotspots`/`timeline`/`tasks`/`task`/
`clean`/`env`/`check`/`setup`/`completions`/`config check`/... ) plus `.bnk`
as a signature alternate filename for the `blink.toml` schema, and a Phase 9
1.0-stabilization pass (actionable errors, grouped `--help`,
`FEATURE_AUDIT.md`, more fixtures/docs). The workspace is now **fourteen
crates**. 200 tests passing, `cargo fmt`/`clippy -D warnings` clean. See
`docs/roadmap.md`'s v0.5/v0.6 sections for what shipped and which spec
collisions were resolved how. **Brand:** Blink's accent is **pink
(`#ff2d8d`)**, not orange — orange belongs to the sibling product **Beacon**
(`v0.6.1` recolored the terminal UI, report, dashboard, placeholder, and
landing page accordingly).
**Published & auto-releasing (currently `0.6.1`):** the repo is public and
on npm as **`@martin-k-m/blink`** (`npm install -g @martin-k-m/blink`). The
unscoped `blink-cli` on npm is an unrelated package; always use the scoped
name. **Releasing is fully automated** via `.github/workflows/release.yml`
on a `v*.*.*` tag: it builds cross-platform binaries, creates the GitHub
release, publishes to npm (`--provenance`, needs the `NPM_TOKEN` secret),
and publishes to GitHub Packages (repo sidebar). To cut a release: bump the
workspace + `packages/blink-cli` versions together, then
`git tag vX.Y.Z && git push origin vX.Y.Z` — no manual `npm publish`.
(Gotcha fixed here once: an `on: release` publish workflow never fired
because releases created by the default `GITHUB_TOKEN` don't emit a
triggering `release` event; the publish now lives inside release.yml.)

**Dependency advisories — resolved.** The 4 Dependabot alerts that had
been open since v0.4 (1 high, 2 moderate, 1 low) came from the **test
fixtures**, not from real dependencies; `f0df4e6` bumped
`flask`/`requests`/`express` off the flagged versions. Separately,
`blink security` was rebuilt to be worth trusting: it used to audit only
the **7 declared** dependencies and reported this repo clean, which was
false reassurance. It now audits the **full resolved `Cargo.lock`** (236
queried packages) and finds 3 transitive advisories — `lru` 0.12.5
(`GHSA-rhfx-m35p-ff5j`, low, via `ratatui`), `number_prefix` 0.4.0
(`RUSTSEC-2025-0119`, unmaintained, via `indicatif`), and `paste` 1.0.15
(`RUSTSEC-2024-0436`, unmaintained, via `ratatui`), matching `cargo
audit`'s three informational warnings exactly. None is fixable by
`cargo update`: two are upstream "unmaintained" notices with no fixed
version, the third needs a `ratatui` release that bumps `lru`. Note that
Blink audits **one ecosystem per invocation** (the project detected at the
given path), so point it at a subdirectory to audit that manifest — e.g.
`blink security examples/react-app` returns 16 npm advisories against the
`vite` 5.2.0 that example pins. See `docs/FEATURE_AUDIT.md`'s "Dependency
& tech-debt notes" and `docs/analysis.md`'s vulnerability-checking
section.

**The "phase 5–8" specs are now shipped (v0.5).** Universal project
intelligence, the daily-driver task runner, the indexing engine, and the
`.bnk` config file all landed — see `docs/roadmap.md`'s v0.5 section.
`docs/planning/phase-5-8-vision.md` is preserved as *historical proposal*
context (it describes the ask, not the delivered result; read the roadmap
for what actually shipped and how the collisions were resolved). Two items
were deliberately deferred and are tracked under v0.7: a real remote
plugin *registry* and `blink self update` — both need a published release /
standing infrastructure to build and verify against, which doesn't exist
yet, so implementing them now would mean unverifiable code.

## Environment setup (this matters — read before running any cargo command)

This has been a **Windows sandbox with no Rust or Node.js on PATH by
default**. If a fresh session can't run `cargo` or `node`, this is why,
and here's the fix:

- **Rust toolchain:** installed via `winget install --id Rustlang.Rustup`.
  The default `x86_64-pc-windows-msvc` host toolchain **does not work in
  this sandbox** — there's no MSVC linker, and git-bash's PATH shadows a
  fake `link.exe` (GNU coreutils' hardlink tool) ahead of any real linker,
  producing baffling "extra operand" errors that look like a linker bug
  but aren't. The fix that worked: install and default to
  `stable-x86_64-pc-windows-gnu` (`rustup toolchain install
  stable-x86_64-pc-windows-gnu && rustup default
  stable-x86_64-pc-windows-gnu`), **and** install a real MinGW toolchain
  via `winget install --id BrechtSanders.WinLibs.POSIX.UCRT` (rustup's own
  bundled "self-contained" linker for the GNU target is incomplete — it's
  missing `as`/other binutils that `dlltool` needs, so builds still fail
  without a full external MinGW install).
- **Node.js:** already installed on this machine at
  `C:\Program Files\nodejs\node.exe`, just not on PATH for tool-invoked
  shells. Needed for `packages/blink-cli`'s install/postinstall scripts.
- **PATH does not persist between tool calls.** Each Bash/PowerShell
  invocation is a fresh shell that doesn't inherit exports from a
  previous call, and registry-level PATH edits (`setx`, `[Environment]::
  SetEnvironmentVariable(..., "User")`) don't take effect in an
  already-running tool-shell host process either. The reliable pattern:
  prefix every cargo-invoking command with the full PATH additions, e.g.:

  ```powershell
  $mingwBin = "C:\Users\comma\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin"
  $env:Path = "$env:USERPROFILE\.cargo\bin;$mingwBin;" + $env:Path
  cargo test --workspace
  ```

  or in Bash: `export PATH="$PATH:/c/Users/comma/.cargo/bin"` (and for
  Node: `export PATH="$PATH:/c/Program Files/nodejs"`) before the actual
  command, every time.
- **Never `git checkout -- <file>` (or any destructive git op) without
  checking `git status`/`git diff` first, and prefer not to at all
  mid-session.** This happened once in this project's history: a
  `git checkout -- crates/blink-cli/src/main.rs` intended to undo a
  one-line test edit instead silently reverted the file to the *last
  commit*, wiping out an entire session's worth of uncommitted subcommand
  wiring. It was caught and manually re-applied, but it cost real time
  and could have been worse. Use the Edit tool to undo specific changes
  instead of git plumbing, especially when there's uncommitted work.

## Verification workflow

Every change in this project's history has been verified the same way
before being considered done:

```sh
cargo fmt --all
cargo fmt --all -- --check          # confirm it's actually clean
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p blink-cli  # release profile catches things dev doesn't
```

Then a manual smoke test of whatever command changed, run against this
repo itself (`blink scan .`, `blink analyze .`, etc. from
`target/debug/` or `target/release/`) — not just unit tests. Several real
bugs in this project were caught only by looking at actual command
output (e.g. a fixture's own explanatory comment accidentally containing
the literal dependency name it was supposed to test as "unused",
defeating the test).

Clean up `.blink/` (the build-cache dir) and any `node_modules/` created
during manual testing before considering a change done — `git status`
should show only intentional source changes.

## Design standards this codebase holds itself to

Documented explicitly in `docs/architecture.md` ("Design decisions worth
knowing about") and `docs/analysis.md`, and worth internalizing before
adding anything:

- **No fabricated numbers or fake output.** Every timing, size, score, or
  count shown anywhere is measured at run time or derived from a
  concrete, checkable fact. Where something is a heuristic (the health
  score, its sub-scores), it's explicitly documented as one, with the
  exact rule/weight table shown.
- **Offline by default.** The only network calls anywhere are the opt-in
  `--online` outdated-package check and `blink security`'s OSV.dev
  lookup. Both are opt-in specifically so `cargo test` never needs
  network access.
- **Real verification over plausible-looking work.** Features in this
  project (the plugin system, the npm installer, the dashboard) were
  proven working end-to-end with actual execution — a real compiled test
  plugin, a real `npm install` + postinstall run, an actual rendered TUI
  frame buffer — not just "the code looks right." Prefer that pattern
  for new work too.

## Where to look for more context

- `docs/architecture.md` — crate-by-crate breakdown and why the
  boundaries are where they are.
- `docs/analysis.md` — exact rules/weights behind every score and check.
- `docs/cli.md` — every command and flag.
- `docs/roadmap.md` — what shipped in which release, what's next.
- `docs/planning/phase-5-8-vision.md` — the not-yet-started future work
  described above, preserved in full.
- `CONTRIBUTING.md` — contributor-facing version of the workflow above.
