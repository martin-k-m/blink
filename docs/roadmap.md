# Roadmap

Blink is being built in phases. Each phase is meant to ship as a usable
release on its own rather than as a stepping stone that only matters once
everything is finished.

## Phase 1 — MVP (shipped in v0.1)

- [x] `blink init` / `blink scan` — manifest-based project detection for
      Rust, TypeScript/JavaScript, and Python projects.
- [x] `blink analyze` — dependency graph, unused/duplicate detection, and
      an opt-in registry-backed outdated check.

## Phase 2 — Developer loop (shipped in v0.1)

- [x] `blink run` — an async dev server with a debounced file watcher.
- [x] File change notifications that rebuild the dependency graph and
      invalidate cache entries for changed files.

## Phase 3 — Build system (partially shipped in v0.1)

- [x] `blink build` — content-hash caching so unchanged files are reported
      as such instead of being reprocessed.
- [ ] An actual optimization/bundling pass (asset minification,
      compression, code splitting). Today `blink build` is a cache-aware
      file scanner, not a bundler — there's nothing here yet to optimize.
- [ ] A persisted performance dashboard across builds.

## Phase 4 — Ecosystem

- [ ] VS Code extension surfacing scan/analyze results inline.
- [ ] A plugin system for framework-specific detection and build steps.
- [ ] Suggestions that go beyond dependency hygiene (e.g. flagging likely
      dead code paths).

If you're picking this up: Phase 3's optimization pass is the biggest
open gap between what the CLI's output implies ("Build") and what it
currently does (cache bookkeeping). See
[`docs/architecture.md`](architecture.md) for how the crates are wired
together before starting there.
