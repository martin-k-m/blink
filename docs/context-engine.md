# The Context Engine

Blink's context engine builds a structured, local **understanding** of a
codebase — and answers questions about it — without a language model and
without sending anything off your machine. This document explains what the
context graph is, how it's built, exactly what is (and isn't) measured, and
how each command uses it.

The guiding rule is the same one the rest of Blink holds itself to: **no
fabricated output.** Every file, symbol, count, and relationship in the
graph is read or resolved from your real files. Where a reference can't be
resolved to a concrete file, Blink drops it rather than inventing an edge.

## Why this exists

AI can generate code quickly. Understanding an *existing* codebase — its
structure, its conventions, how its pieces depend on one another — is still
the hard part, for people and for tools. The context engine is the missing
layer: a fast, deterministic, offline model of a project that both a
developer and their tooling can rely on.

## The context graph

`blink context` (and every other context command) builds a `ContextGraph`,
assembled from three things Blink already knows about a project:

1. **Identity** — from `blink-core`'s detector: name, language, framework,
   package manager, workspace flag.
2. **Files & symbols** — from `blink-index`'s incremental index: every
   file's size, line count, language, and top-level symbols.
3. **Dependencies** — the direct dependencies declared in the manifest.

On top of that, the engine adds two things the index doesn't track:

- **Areas** — files grouped into logical areas (see [Areas](#areas)).
- **References** — resolved file→file edges (who imports whom), plus, in a
  Cargo workspace, cross-crate edges (see [References](#references)).

The result is one serializable model:

```
ContextGraph
├── project        name · language · framework · package manager · workspace?
├── stats          files · source files · lines · symbols · size
├── config         .bnk/blink.toml present? · [context] settings
├── areas[]        path · files · lines · symbols · size · languages
├── dependencies[] name · version · dev?
├── commands[]     name · command · source
├── files[]        path · area · language · lines · size · symbols[]
└── references[]   from-file → to-file
```

## Areas

An *area* is a directory grouping — the one interpretive choice in the
graph (a presentation grouping, never a measured number). The rule is fixed
and deterministic:

- A top-level file (e.g. `README.md`) belongs to `(root)`.
- A file directly under a **container** directory groups by that container:
  `src/main.rs` → `src`.
- A file nested under a container groups by container + next segment:
  `src/auth/login.rs` → `src/auth`; `crates/blink-core/src/lib.rs` →
  `crates/blink-core`.
- Anything else groups by its top-level directory.

Container directories are the conventional source roots: `src`, `lib`,
`libs`, `app`, `apps`, `crates`, `packages`, `pkg`, `internal`, `cmd`,
`source`, `modules`. Every count *inside* an area (files, lines, symbols,
size) is measured; only the grouping is a rule.

Areas are ranked for display by symbol density (most symbols first, then
lines), which surfaces a project's substantial code areas ahead of its
config and docs directories.

## References

References are the edges that make this a *graph* rather than a list.
Resolution is deliberately conservative and per-language — it mirrors
`blink-index`'s symbol scanner, which prefers to miss a declaration over
inventing one.

| Language | Resolved | How |
| --- | --- | --- |
| TypeScript / JavaScript | Relative imports | `import … from './x'`, `require('./x')`, `import('./x')`, side-effect `import './x'` — resolved against the importing file's directory, trying `.ts/.tsx/.js/.jsx/.mjs/.cjs` and `/index.*`. Bare specifiers (`react`, `@scope/pkg`) are recorded as external, not internal edges. |
| Python | Absolute & relative imports | `import a.b`, `from a.b import c`, `from . import x`, `from ..pkg import y` — resolved to `a/b.py` or `a/b/__init__.py`, with leading dots walking up package levels. |
| Rust | `mod` declarations | `mod name;` / `pub mod name;` resolved to the sibling `name.rs` or `name/mod.rs`, using Rust-2018 module-directory rules. |
| Rust (workspace) | Cross-crate paths | Any `<crate>::…` reference — in a `use` or an inline path like `blink_core::Foo` — is resolved to that crate's `src/lib.rs` (or `main.rs`) when the crate is a workspace member. Crate names are read from each `Cargo.toml`'s `[package].name`. |
| Go | — | Go imports are package-path based and can't be mapped to files without the module path, so no internal edges are recorded. Symbols are still indexed. |

**An import that cannot be resolved to a file already in the index is never
turned into an edge.** Unresolvable imports become "external" specifiers
(useful in `blink explain`) but never fabricate a relationship.

Aggregating file→file references by area yields the **area dependency
graph** that `blink map` shows and `blink export --format graph` renders as
Mermaid.

## Commands

### `blink context`

Builds the graph and prints an understanding report: identity, measured
statistics, the most significant areas (ranked by symbols), and the
discovered commands. `--json` emits the entire graph.

### `blink query <text>`

Structured, **local** search over the graph — not AI, no inference. The
query is tokenized (splitting `camelCase`), common question/stop words are
dropped (`where`, `is`, `the`, …), and the remaining terms are ranked
against area paths, file paths, symbol names, dependency names, and command
names. Results are grouped by kind. So `blink query "where are the API
routes"` searches for `api` + `routes` and surfaces the matching areas,
files, and symbols — relationships and structure, not just a text grep.

### `blink explain <file>`

Explains one file using **only real signals**:

- its own leading doc comment (module doc / JSDoc / docstring), verbatim;
- the top-level symbols it declares;
- the project files it imports (resolved) and the external packages it
  imports;
- the project files that import it.

There is deliberately no invented prose about what the file "does" — no
guessed "responsibilities". Every line is read from the file or the index.

### `blink map`

The architecture view: areas ranked by symbols, and the area→area
dependency edges derived from resolved references. `--format` selects
`terminal` (default), `markdown`, `json`, or `graph` (a Mermaid diagram).

### `blink export`

Serializes the whole graph. `--format` selects `json` (default), `yaml`,
`markdown` (a readable project document), or `graph` (Mermaid). Writes to
stdout, or to `--output <file>`; the conventional filenames are
`blink-context.json` / `.yaml` / `.md` / `.mmd`. YAML is produced by a
small internal block-style emitter, so exporting takes on no external YAML
dependency.

## Configuration

The `[context]` table in `blink.toml` / `.bnk` tunes the engine:

```toml
[context]
enabled = true          # set false to disable the context commands
include = ["src", "lib"] # limit the graph to these roots (default: whole project)
```

`include` roots match on a path-segment boundary, so `"src"` covers
`src/main.rs` but not `srcgen/x.rs`. `blink config check` reports the
effective settings.

## Design notes

- **Local, offline, deterministic.** The engine makes no network calls and
  produces the same graph for the same bytes on disk. It reuses the warm
  `blink-index` rather than rescanning.
- **Three crates, one model.** `blink-context` builds the graph,
  `blink-query` searches it, and `blink-export` serializes it — three
  separable concerns over one shared, serde-serializable model.
- **Conservative by construction.** Reference resolution never guesses.
  This keeps the graph trustworthy as a foundation for tooling: an edge in
  the graph is an edge in the code.
