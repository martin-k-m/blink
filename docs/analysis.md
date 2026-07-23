# Dependency Analysis

`blink analyze` runs everything below and prints a report; `blink analyze --json`
prints the same data as JSON instead. This document describes exactly what
each part measures, including where the measurements are approximations —
Blink would rather tell you a number is a heuristic than present it as more
precise than it is.

## Dependency graph & counts

- **Direct dependencies** come straight from the project's manifest
  (`Cargo.toml` / `package.json` / `requirements.txt`).
- **Transitive dependencies** are derived as `(total resolved packages in the
  lockfile) - (direct dependencies)`. This is an approximation: it assumes
  every resolved package that isn't a direct dependency is transitive,
  without walking the actual dependency edges. That holds for the vast
  majority of projects and avoids reimplementing a package manager's
  resolver just to count nodes. If there's no lockfile yet, transitive is
  reported as unavailable rather than guessed.

## Unused dependencies

Blink scans every source file matching the project's language (`.rs` for
Rust; `.ts`/`.tsx`/`.js`/`.jsx`/`.mjs`/`.cjs`/`.vue`/`.svelte` for JS/TS;
`.py` for Python) and checks whether each *runtime* dependency's name
appears anywhere in that text.

Two things worth knowing:

- **Dev dependencies are excluded.** Tools like `typescript` or `eslint`
  are typically invoked from config files or the CLI, not imported — scanning
  for them produces false positives.
- **This is a substring match, not an AST-aware import check.** A
  dependency name appearing in a comment or a string literal counts as
  "referenced," which can produce false negatives (a genuinely unused
  package gets missed). It cannot produce false positives in the other
  direction for real code, since an actual `import`/`use` always contains
  the name. A real import-graph analysis is a natural v0.3+ improvement
  (see [`docs/roadmap.md`](roadmap.md)).

## Duplicate versions

Blink parses the project's lockfile (`Cargo.lock` or `package-lock.json`)
and groups resolved packages by name. Any name resolving to more than one
distinct version is reported as a duplicate. This requires a lockfile to
exist — without one, there's nothing resolved yet to compare.

## Large dependencies & "Largest Packages"

Blink measures each direct dependency's installed size on disk:

- **JS/TS:** the size of `node_modules/<name>`, once `npm install` (or
  equivalent) has actually run.
- **Rust:** the size of the crate's extracted source under
  `$CARGO_HOME/registry/src/*/<name>-<resolved-version>`, once `cargo
  build` has downloaded it. This requires the *resolved* version from
  `Cargo.lock`, since the manifest's version requirement (e.g. `"1"`)
  essentially never matches the cache directory name exactly.

Dependencies at or above 5 MiB are flagged as "large" (an issue). The
report's "Largest Packages" table shows the top 5 by size regardless of
that threshold. Both are empty if dependencies haven't been installed or
built locally — there's nothing on disk yet to measure.

## Outdated dependencies (`--online`)

Off by default. With `--online`, Blink queries crates.io / the npm
registry for each dependency's latest published version and compares it
against what's declared. This is the only network call anywhere in Blink.

## Health score

A single 0-100 number, rendered as a 10-segment bar (`█████████░ 92%`).
It's a simple weighted deduction from 100, **not** a rigorous metric:

| Finding                  | Penalty per occurrence |
| ------------------------- | ----------------------|
| Unused dependency          | -5                    |
| Duplicate-version package  | -4                    |
| Large dependency           | -3                    |
| Outdated package (`--online` only) | -2            |

The score floors at 0. Treat it as an at-a-glance signal for "does this
project need attention," not a precise measurement — two projects with
very different real-world health can land on the same score.

`healthy_count` (shown in the Dependencies table) is `direct dependencies -
(unused + duplicates + outdated)`, floored at zero. Because duplicate
versions are counted per *lockfile* name rather than per direct dependency,
a project with many duplicates found deep in its transitive tree can show
`0` healthy direct dependencies even though none of those direct
dependencies are themselves the problem. Read it alongside the "Potential
Issues" section rather than in isolation.

## Project health sub-scores (`blink health`)

`blink health` breaks health into three independently measurable
0-100 sub-scores, averaged for an overall score. Like the analyze health
score above, these are explicit, checkable rules — not invented numbers:

**Dependencies** — the same score `blink analyze`'s health score computes
(see above).

**Configuration** — 25 points each for:
- a `blink.toml` existing
- a lockfile existing (`Cargo.lock` / `package-lock.json` /
  `pnpm-lock.yaml` / `yarn.lock`)
- a `.gitignore` existing
- a `README.md` (or `README`/`readme.md`) existing

**Code Organization** — measured from directory presence:
- 40 points for a `src/` or `crates/` directory
- 30 points for a `tests/`, `test/`, or `__tests__/` directory
- 30 points for a `docs/` directory

Each sub-score's suggestions ("Add a .gitignore", "Add a tests/
directory", ...) come directly from whichever checks didn't pass.

## Recommendation rules (`blink recommend`)

`blink recommend` groups findings into three categories, each rule
checking one concrete fact:

- **Performance** — whether `[optimization].cache` is enabled in
  `blink.toml` (defaults to enabled if there's no config); whether any
  large dependencies were found.
- **Maintenance** — unused dependencies, duplicate versions, and (with
  `--online`) outdated packages.
- **Security** — known vulnerabilities (see below). Without `--online`,
  reported as "unknown" rather than assumed clean.

## Vulnerability checking (`blink security`, `--online`)

Queries [OSV.dev](https://osv.dev) — Google's free, open Open Source
Vulnerabilities database — via its batch API. Ecosystem mapping: Rust →
`crates.io`, JS/TS → `npm`, Python → `PyPI`.

### What gets audited

The audit set is the **fully resolved dependency graph**, read from the
lockfile belonging to the project's ecosystem:

| Lockfile            | Resolved versions | Dependency edges |
| ------------------- | ----------------- | ---------------- |
| `Cargo.lock`        | yes               | yes              |
| `package-lock.json` | yes               | yes              |
| `yarn.lock`         | yes               | no               |
| `pnpm-lock.yaml`    | yes               | no               |

Only the lockfiles matching the detected ecosystem are considered, so a
`Cargo.lock` sitting beside a JS project is never queried against `npm`.
Cargo workspace members (`[[package]]` entries with no `source`) are the
project itself and are excluded — they aren't published packages, so
there's nothing to look up. Each distinct `(name, version)` pair is queried
once, in batches of 500.

**Rust matters here.** Before this landed, `blink security` queried only
the versions *declared* in the manifest — for this repository, 7 packages
instead of the 236 the lockfile resolves. Nearly every real Rust advisory
lands on a transitive dependency, so a declared-only audit reported clean
projects that weren't.

When **no lockfile exists** the audit falls back to declared manifest
versions (with semver range operators stripped) and says so explicitly:
transitive dependencies are not covered by that result. Python always
takes this path, since Blink reads no Python lockfile format.

### How findings are classified

- **Direct** — the package name appears in the project manifest, or in the
  lockfile's recorded direct-dependency set (for Cargo, the union of every
  workspace member's dependencies; for npm, the root entry's
  `dependencies`/`devDependencies`/`optionalDependencies`).
- **Transitive** — everything else.
- **Dependency path** — the shortest chain from any direct dependency to
  the affected package, found by multi-source breadth-first search over the
  lockfile's recorded edges (`ratatui → lru`). Name-keyed, so a package
  resolved at two versions shares one node. When the lockfile records no
  edges Blink shows no path at all rather than guessing one.

### Severity

Severity comes from the `database_specific.severity` field of the OSV
record — that is, the GitHub Advisory Database rating GitHub itself shows
(`critical` / `high` / `moderate` / `low`). Advisories without one, which
includes most RUSTSEC-only records such as "crate is unmaintained"
notices, are reported **unrated**. Blink never infers a severity.

OSV often stores one advisory under several IDs (a `GHSA-...` record and
its `RUSTSEC-...` alias). Those are collapsed into a single advisory using
OSV's own `aliases` list, preferring the record that carries a severity,
so counts don't double up. The summary line counts *distinct* advisories,
so one advisory affecting two packages counts once.

### "Clean" means clean

The report distinguishes four outcomes, and only the first is a pass:

| Outcome              | Meaning                                                 | Exit |
| -------------------- | ------------------------------------------------------- | ---- |
| completed            | Every audited package was checked. Findings may be zero. | `0`  |
| source unavailable   | OSV.dev couldn't be reached or returned garbage.         | `1`  |
| unsupported ecosystem| No OSV ecosystem for this project's language.            | `0`  |
| nothing to audit     | No dependencies and no lockfile.                         | `0`  |

A failed lookup is never rendered as "no vulnerabilities found" — the
command prints that nothing was verified and exits non-zero. Likewise
`blink recommend`/`blink ci` report the security verdict as *unknown*, not
*ok*, when the audit couldn't run.

Advisory records are fetched individually so findings can carry a real
summary and severity. If a record can't be fetched, the ID is still
reported — just without metadata — and the report says the detail set is
incomplete.

## The global analysis cache

Beyond the per-project `.blink/cache.json` build cache (see
[`docs/configuration.md`](configuration.md)), Blink keeps a global,
per-user cache in the platform cache directory (`~/.cache/blink` on
Linux, `~/Library/Caches/blink` on macOS, `%LOCALAPPDATA%/blink` on
Windows). Commands that call the analyzer without `--online`
(`analyze`, `deps`, `health`, `recommend`, `ci`, `report`) check this
cache first: if a content-hash snapshot of the project's files matches
what was cached, the previous `AnalysisReport` is reused instead of
recomputed.

`--online` results are never cached — registry/vulnerability data can go
stale independent of whether the project's files changed, so caching that
result risks reporting an outdated "up to date" or "no vulnerabilities"
verdict. `blink watch`'s change-triggered refreshes also bypass the cache
(the files just changed, so a cache hit would be unusual, and writing to
the global cache on every keystroke-triggered save is unnecessary I/O for
a live loop).

`blink analyze` prints "Using cache" when it served a cached result, and
its "Analysis time" reflects the actual cache-lookup latency for that
run — not the (possibly much larger) time the original, uncached run
took. `blink benchmark` measures both explicitly.

## The project index

`blink index` builds a per-project index (`.blink/index.json`) recording,
for every non-ignored file: its size, SHA-256 hash, modification time,
language (by extension), line count, and top-level symbols. It's the
foundation for `search`, `symbols`, `hotspots`, `status`, and the
statistics in `inspect`/`docs`, and it exists so those commands don't
re-scan and re-hash the whole tree every run.

A **refresh** is incremental: a file whose size *and* mtime both match its
stored record is left untouched; only changed, added, or removed files are
re-hashed and re-parsed (in parallel via rayon). `blink index` reports the
exact counts (`added`/`updated`/`removed`/`unchanged`) so the saving is
visible, not asserted. `--rebuild` forces a full rebuild. This is distinct
from the global analysis cache above: that caches a whole-project
*analysis result* keyed by a content snapshot (any change invalidates all
of it); the index tracks *per-file* state and updates only what changed.

**Symbol extraction is a conservative line scanner, not a parser.** It
finds top-level `fn`/`struct`/`enum`/`trait`/`class`/`interface`/`type`/
`def`/`func` declarations by keyword and reads the following identifier,
across Rust, Python, TypeScript, JavaScript, and Go. It doesn't understand
scope, macros, or generics, and it deliberately errs toward *missing* an
oddly formatted declaration rather than inventing one — so a reported
symbol is always a real declaration, even though the list may not be
exhaustive.

`[index].enabled = false` in `blink.toml`/`.bnk` makes index-backed
commands build a throwaway in-memory index instead of persisting one;
`[index].auto_update = false` stops them refreshing/saving before they run.

## The optimize score

`blink optimize` reports a 0–100 score. The rule is deliberately simple
and documented rather than precise: it starts at 100 and deducts a fixed
**8 points per warning category**, clamped to `[0, 100]`. There are six
categories, each `✓` (good) or `⚠` (warning) based on one concrete,
measured condition:

| Category          | Warns when                                                                 |
| ----------------- | ---------------------------------------------------------------------------|
| Dependencies      | the analyzer found unused, duplicate-versioned, or oversized packages.     |
| Project Structure | some directory holds more than 300 files (heuristic; flagged for splitting). |
| Duplicate Files   | the index found two or more byte-identical files.                          |
| Tests             | no test files were detected (paths under `tests`/`__tests__`, or `*.test.*`/`*.spec.*`/`*_test.*`/`test_*` names — a heuristic). |
| Documentation     | `README` or `CONTRIBUTING` is missing.                                     |
| Configuration     | `.gitignore` or a CI configuration is missing.                             |

The per-category findings and their suggestions are the substance; the
score is an at-a-glance roll-up of them. **No build-speed improvement is
ever claimed here**, because `optimize` measures none — it reports
conditions, not benchmarks.

## JSON export (`--json`)

```json
{
  "project": "my-app",
  "type": "React + TypeScript",
  "files": 2431,
  "dependencies": { "direct": 12, "transitive": 183 },
  "health": { "score": 92, "healthy_packages": 9 },
  "issues": ["Duplicate package versions detected (3)"],
  "suggestions": ["Deduplicate lodash (2 versions resolved: 3.10.1, 4.17.21)"],
  "analysis_time_ms": 7
}
```

Every field here is the same data shown in the terminal report — `--json`
doesn't compute anything extra, it just skips formatting. `transitive`,
`issues`, and `suggestions` are absent-safe: `transitive` may be `null`,
and the arrays may be empty on a clean project.
