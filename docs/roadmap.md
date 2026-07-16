# Roadmap

Blink is being built in phases, each shipped as a usable release rather
than a stepping stone that only matters once everything is finished. This
file tracks releases (`v0.1`, `v0.2`, ...); it doesn't reuse "Phase N"
numbering on its own, since that turned out to drift from how the work
actually shipped — see the note at the bottom.

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

## v0.5 — Blink Runtime (planned)

This is where `blink build` stops being cache bookkeeping and starts being
a real build tool:

- [ ] An actual optimization/bundling pass (asset minification,
      compression, code splitting). Today's `blink build` scans and hashes
      files — there's nothing here yet to optimize.
- [ ] A persisted performance dashboard across builds (distinct from
      `blink dashboard`'s live single-run view).
- [ ] AST-aware unused-dependency detection, replacing the current
      substring scan (see the known limitation in `docs/analysis.md`).

## v0.6 — Ecosystem (planned)

- [ ] VS Code extension surfacing scan/analyze results inline.
- [ ] A plugin *registry* — `blink plugins install` currently only copies
      a local file; there's no `blink plugins install <name>` fetching
      from anywhere, because no registry exists yet.
- [ ] Suggestions that go beyond dependency hygiene (e.g. flagging likely
      dead code paths, once the AST-aware analysis above lands).

## Beyond v0.6 — proposed, not started

The project owner has written detailed proposals for several more
phases: universal project intelligence + a workflow-optimization engine
(`blink inspect`/`optimize`/`doctor`/`duplicates`/...), a daily-driver
task runner and environment manager (`blink run` as a task runner,
`setup`/`check`/`clean`/`env`/`update`/self-update/shell completion),
a performance-focused indexing engine (`blink index`/`search`/`symbols`/
`hotspots`), and a Blink-native project config file (proposed as `.bnk`,
pending a naming-collision check against `blink.toml`/`.blink`, both of
which are already taken by this project's own conventions).

**None of this is implemented.** The full original specs, plus notes on
where each proposal overlaps or conflicts with what's already shipped
(e.g. `blink run` already means something different; `blink doctor` is
specified twice with different scope; the proposed `.bnk` fallback names
collide with existing `blink.toml`/`.blink/`), are preserved verbatim in
[`docs/planning/phase-5-8-vision.md`](planning/phase-5-8-vision.md). Read
that file — not just this summary — before starting any of it; it flags
several design decisions (command-name collisions, config-file
consolidation, in-process vs. subprocess plugin API) that need resolving
deliberately rather than defaulting to whatever's easiest to type first.

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
