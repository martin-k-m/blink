# Roadmap

> **Release version vs. milestone labels.** Blink's first *public release*
> is **`0.1.0` (early alpha)** — that's the number in `Cargo.toml`, on npm,
> and from `blink --version`. The `v0.1`–`v0.5` headings below are internal
> *development milestones* (chunks of work that built up to the alpha), not
> separate published releases. Don't read "v0.5" here as a release version.

Blink was built in phases, each a usable chunk of work rather than a
stepping stone that only matters once everything is finished. The headings
below track those development milestones.

## v0.1 — MVP + developer loop (shipped)

- [x] `blink init` / `blink scan` — manifest-based project detection for
      Rust, TypeScript/JavaScript, and Python projects.
- [x] `blink analyze` — dependency graph, unused/duplicate detection, and
      an opt-in registry-backed outdated check.
- [x] `blink run` — an async dev server with a debounced file watcher;
      saving a file rebuilds the dependency graph and invalidates cache
      entries for the changed files.
- [x] `blink build` — content-hash caching so unchanged files are reported
      as such instead of being reprocessed.

## v0.2 — Intelligent project analysis (shipped)

- [x] Split `blink-parser` (manifest/lockfile *format* parsing) and
      `blink-report` (output formatting) out of `blink-core`/`blink-analyzer`,
      so file-format knowledge and presentation logic each live in one place.
- [x] Direct/transitive dependency counts, largest-installed-package ranking
      (now measured for Rust too, via the local Cargo registry cache — not
      just JS/TS's `node_modules`).
- [x] A documented, heuristic health score (`docs/analysis.md`) rendered as
      a progress bar.
- [x] `blink analyze --json` for machine-readable output.
- [x] `indicatif` spinners (skipped automatically on non-interactive
      output) and `comfy-table` tables for a more polished terminal report.
- [x] Fixture projects under `tests/fixtures/` purpose-built to exercise
      specific analyzer behavior (duplicate versions, unused dependencies,
      Python detection) rather than relying solely on the showcase
      projects in `examples/`.

## v0.3 — Installable CLI (shipped)

The "clone the repo to use it" → "install it like a real tool" milestone:

- [x] `blink scan --verbose` — resolved path, matched manifest file, and
      the effective ignore-directory list.
- [x] `[project].ignore` in `blink.toml` — extra directories skipped
      during scans/builds, on top of the built-in list.
- [x] Expanded detection: Vite (only when no other framework was
      detected — it's a build tool, not competing with React/Vue/etc.),
      Python virtualenv presence, Cargo `[workspace]` detection.
- [x] `packages/blink-cli` — the npm distribution package. `npm install
      -g blink-cli` downloads the matching platform binary, verifies its
      checksum, and installs a `blink` shim on `PATH`.
- [x] `.github/workflows/release.yml` builds and checksums binaries for
      linux-x64, linux-arm64, macos-x64, macos-arm64, and windows-x64 on
      every `v*.*.*` tag.

## v0.4 — Intelligent platform (shipped)

- [x] `blink deps` / `blink health` / `blink recommend` — focused views
      over the same analysis engine, rather than duplicated logic.
- [x] `blink watch` — analysis-only live reload, without `run`'s dev
      server.
- [x] `blink ci` — pipeline-friendly exit codes (0 pass / 1 warnings / 2
      failure).
- [x] `blink security` — OSV.dev vulnerability lookups (opt-in, since it's
      a network call).
- [x] `blink report --json|--markdown|--html` — full document export, not
      just the terminal view.
- [x] A global, per-user analysis cache (separate from the project-local
      build cache) so repeated `analyze`/`deps`/`health`/`ci` runs against
      an unchanged project reuse the previous result. `blink benchmark`
      measures the real difference.
- [x] `blink-plugin` — a real subprocess-based plugin system (`blink
      <name>` runs `blink-<name>` if installed), in the style of
      `cargo`/`git`. No remote registry, no dynamic loading — see
      [`docs/plugins.md`](plugins.md) for why.
- [x] `blink dashboard` — an interactive `ratatui` terminal UI with live,
      file-watcher-triggered refresh.

## v0.5 — Project intelligence, indexing & workflow engine (shipped)

The "Beyond v0.6" proposals (originally "phases 5–8") landed here, built
on a new incremental index and a fact-driven workflow engine rather than
by piling on shallow commands. Two new crates — `blink-index` and
`blink-workflow` — plus 21 new subcommands:

- [x] **Indexing** (`blink-index`): a per-project `.blink/index.json`
      tracking each file's size/hash/mtime/language/lines/symbols. Refreshes
      incrementally (only changed files re-hashed, rayon-parallel) and powers
      `index`, `status`, `search`, `symbols`, `hotspots`, and the stats in
      `inspect`/`docs`. Symbol extraction spans Rust/Python/TS/JS/Go.
- [x] **Project intelligence**: `inspect` (what is this / how to run it /
      where to start), `optimize` (rule-based score + suggestions),
      `duplicates`, `doctor` (env/runtime diagnostics), `filesystem`,
      `config-audit`, `docs`.
- [x] **Daily workflow**: `tasks`/`task` (discovery + running across
      package.json/Makefile/justfile/Cargo/`[commands]`), `profile`, `clean`,
      `env`, `check` (real local toolchain), `setup`, `completions`.
- [x] **`.bnk`** as a signature alternate filename for the exact
      `blink.toml` schema, with new `[commands]`/`[index]`/`[profiles]`/
      `[scan]`/`[report]`/`[plugins.*]` tables and a `blink config check`
      validator.

Collision resolutions from the original specs (see
[`docs/planning/phase-5-8-vision.md`](planning/phase-5-8-vision.md)):
`blink run` stayed the dev server (task-running is `blink task`); the two
`doctor` specs were reconciled into one command; `check` runs the
toolchain while `ci` keeps its exit-code contract; `.bnk` extends the
existing config rather than introducing a parallel format.

**Deferred, deliberately:** a real remote plugin *registry* and `self
update` (both need standing infra / a published release to verify against,
which doesn't exist yet — see the note below), and true partial
incremental *analysis* (the index tracks per-file state, but `optimize`/
`analyze` still recompute their whole result).

## v0.6 — Blink Runtime (planned)

This is where `blink build` stops being cache bookkeeping and starts being
a real build tool:

- [ ] An actual optimization/bundling pass (asset minification,
      compression, code splitting). Today's `blink build` scans and hashes
      files — there's nothing here yet to optimize.
- [ ] A persisted performance dashboard across builds (distinct from
      `blink dashboard`'s live single-run view).
- [ ] AST-aware unused-dependency detection, replacing the current
      substring scan (see the known limitation in `docs/analysis.md`).

## v0.7 — Ecosystem (planned)

- [ ] VS Code extension surfacing scan/analyze results inline.
- [ ] A plugin *registry* — `blink plugins install` currently only copies
      a local file; there's no `blink plugins install <name>` fetching
      from anywhere, because no registry exists yet.
- [ ] `blink self update` — check the latest release, verify a checksum,
      swap the binary. Needs a published release to build and verify
      against (there isn't one yet), which is why it wasn't done in v0.5.
- [ ] Suggestions that go beyond dependency hygiene (e.g. flagging likely
      dead code paths, once the AST-aware analysis above lands).

## The original "phase 5–8" proposals

The universal-intelligence / workflow-engine / indexing / `.bnk`
proposals the owner wrote (originally "phases 5–8") **shipped in v0.5
above.** The full original specs, plus the inline notes on where each
proposal overlapped or conflicted with already-shipped behavior, are
preserved verbatim in
[`docs/planning/phase-5-8-vision.md`](planning/phase-5-8-vision.md) for
historical context — but that document describes the *proposal*, not the
delivered result. For what actually shipped and how the flagged collisions
were resolved, read the v0.5 section above; the two items intentionally
left out (a real plugin registry and `self update`) are tracked under v0.7.

## A note on phase numbering

Earlier planning used "Phase 1–4" for, respectively: CLI+scanner,
dev-server+watcher, build caching, and ecosystem work. In practice the
first two shipped together in v0.1, "Phase 2" was reused for the
intelligent-analysis work that became v0.2, and "Phase 3"/"Phase 4" were
reused again for the installable-platform work that became v0.3/v0.4
(installable CLI, then dashboard/plugins/security/caching/CI/benchmarking).
Rather than keep patching numbering that's drifted repeatedly, this file
tracks by release. If you're looking for "Phase 3" or "Phase 4" from an
earlier conversation, that's v0.3/v0.4 above; the original "Phase 3:
Blink Runtime" content (a real bundler) is now v0.5.
