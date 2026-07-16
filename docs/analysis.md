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
Vulnerabilities database — via its batch API, one query per declared
dependency at its declared version. Ecosystem mapping: Rust → `crates.io`,
JS/TS → `npm`, Python → `PyPI`.

Blink reports the vulnerability IDs OSV returns (e.g. `GHSA-...`,
`RUSTSEC-...`) without fetching or summarizing the underlying advisory
text — look an ID up directly at `osv.dev/vulnerability/<id>` for details.
This is the only check besides the outdated-package lookup that touches
the network, and it's always opt-in.

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
