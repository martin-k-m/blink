//! Conservative import/reference extraction.
//!
//! For each supported language this finds the files a source file references
//! and resolves them to *actual files present in the index*. It deliberately
//! prefers to miss a reference over inventing one: an import that cannot be
//! resolved to a real project file becomes an "external" specifier (useful for
//! `blink explain`) but never a fabricated internal edge. This mirrors
//! `blink-index`'s conservative symbol scanner — no full parser, no guessing.

use std::collections::HashSet;

use blink_index::FileLang;

/// The result of scanning one file's imports.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ScanResult {
    /// Project-relative paths (present in the index) this file references.
    pub internal: Vec<String>,
    /// Bare/package or unresolved specifiers this file imports.
    pub external: Vec<String>,
}

/// Scan `text` (a file of language `lang` at project-relative path `importer`)
/// for references, resolving internal ones against `paths` (the set of all
/// project-relative file paths in the index).
pub fn scan(lang: FileLang, text: &str, importer: &str, paths: &HashSet<String>) -> ScanResult {
    let mut internal: Vec<String> = Vec::new();
    let mut external: Vec<String> = Vec::new();

    match lang {
        FileLang::TypeScript | FileLang::JavaScript => {
            scan_ts_js(text, importer, paths, &mut internal, &mut external)
        }
        FileLang::Python => scan_python(text, importer, paths, &mut internal, &mut external),
        FileLang::Rust => scan_rust(text, importer, paths, &mut internal, &mut external),
        // Go imports are package-path based (not file paths) and can't be
        // resolved to files without the module path; record none rather than
        // guess. Symbols are still indexed.
        FileLang::Go => {}
    }

    dedup_sorted(&mut internal);
    dedup_sorted(&mut external);
    ScanResult { internal, external }
}

fn dedup_sorted(v: &mut Vec<String>) {
    v.sort();
    v.dedup();
}

/// Directory portion of a project-relative path (`""` for a top-level file).
fn parent_dir(path: &str) -> &str {
    match path.rsplit_once('/') {
        Some((dir, _)) => dir,
        None => "",
    }
}

/// Resolve a `/`-separated relative `spec` against `base_dir`, collapsing `.`
/// and `..`. Returns `None` if it walks above the project root.
fn resolve_relative(base_dir: &str, spec: &str) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    if !base_dir.is_empty() {
        parts.extend(base_dir.split('/'));
    }
    for seg in spec.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            s => parts.push(s),
        }
    }
    Some(parts.join("/"))
}

// --- TypeScript / JavaScript ------------------------------------------------

const TS_JS_EXTS: &[&str] = &["ts", "tsx", "js", "jsx", "mjs", "cjs"];

fn scan_ts_js(
    text: &str,
    importer: &str,
    paths: &HashSet<String>,
    internal: &mut Vec<String>,
    external: &mut Vec<String>,
) {
    let base = parent_dir(importer);
    for spec in ts_js_specifiers(text) {
        if spec.starts_with('.') {
            if let Some(hit) = resolve_ts_js_relative(base, &spec, paths) {
                internal.push(hit);
            }
        } else {
            external.push(package_root(&spec));
        }
    }
}

/// Extract every module specifier from `from "..."`, `import "..."`,
/// `require("...")`, and `import("...")` forms.
fn ts_js_specifiers(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
            continue;
        }
        // `... from '<spec>'`
        if let Some(idx) = line.find(" from ") {
            if let Some(spec) = first_quoted(&line[idx + 6..]) {
                out.push(spec);
                continue;
            }
        }
        // `require('<spec>')` / `import('<spec>')`
        for marker in ["require(", "import("] {
            if let Some(idx) = line.find(marker) {
                if let Some(spec) = first_quoted(&line[idx + marker.len()..]) {
                    out.push(spec);
                }
            }
        }
        // Side-effect import: `import '<spec>';`
        let after_import = trimmed
            .strip_prefix("import ")
            .or_else(|| trimmed.strip_prefix("import"));
        if let Some(rest) = after_import {
            let rest = rest.trim_start();
            if rest.starts_with('\'') || rest.starts_with('"') {
                if let Some(spec) = first_quoted(rest) {
                    out.push(spec);
                }
            }
        }
    }
    out
}

/// The first single- or double-quoted string in `s`.
fn first_quoted(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let start = bytes.iter().position(|&b| b == b'\'' || b == b'"')?;
    let quote = bytes[start];
    let rest = &s[start + 1..];
    let end = rest.find(quote as char)?;
    Some(rest[..end].to_string())
}

fn resolve_ts_js_relative(base: &str, spec: &str, paths: &HashSet<String>) -> Option<String> {
    let target = resolve_relative(base, spec)?;
    // Exact path (a specifier that already includes its extension).
    if paths.contains(&target) {
        return Some(target);
    }
    // `<target>.<ext>`
    for ext in TS_JS_EXTS {
        let cand = format!("{target}.{ext}");
        if paths.contains(&cand) {
            return Some(cand);
        }
    }
    // `<target>/index.<ext>`
    for ext in TS_JS_EXTS {
        let cand = format!("{target}/index.{ext}");
        if paths.contains(&cand) {
            return Some(cand);
        }
    }
    None
}

/// The installed-package name of a bare specifier: `react/jsx` -> `react`,
/// `@scope/pkg/sub` -> `@scope/pkg`.
fn package_root(spec: &str) -> String {
    if let Some(rest) = spec.strip_prefix('@') {
        let mut it = rest.splitn(3, '/');
        let scope = it.next().unwrap_or("");
        let name = it.next().unwrap_or("");
        if name.is_empty() {
            format!("@{scope}")
        } else {
            format!("@{scope}/{name}")
        }
    } else {
        spec.split('/').next().unwrap_or(spec).to_string()
    }
}

// --- Python -----------------------------------------------------------------

fn scan_python(
    text: &str,
    importer: &str,
    paths: &HashSet<String>,
    internal: &mut Vec<String>,
    external: &mut Vec<String>,
) {
    let base = parent_dir(importer);
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("from ") {
            // `from <module> import ...`
            let module = rest.split_whitespace().next().unwrap_or("");
            if module.is_empty() {
                continue;
            }
            resolve_python_module(module, base, paths, internal, external);
        } else if let Some(rest) = t.strip_prefix("import ") {
            // `import a.b.c`, possibly comma-separated with `as` aliases.
            for part in rest.split(',') {
                let module = part.split_whitespace().next().unwrap_or("");
                if !module.is_empty() {
                    resolve_python_module(module, base, paths, internal, external);
                }
            }
        }
    }
}

fn resolve_python_module(
    module: &str,
    base: &str,
    paths: &HashSet<String>,
    internal: &mut Vec<String>,
    external: &mut Vec<String>,
) {
    // Leading dots denote a relative import; each dot after the first walks up
    // one package level.
    let dots = module.chars().take_while(|&c| c == '.').count();
    let name = &module[dots..];
    let segs: Vec<&str> = if name.is_empty() {
        Vec::new()
    } else {
        name.split('.').collect()
    };

    if dots > 0 {
        // Relative: start from `base`, go up `dots - 1` levels.
        let mut parts: Vec<&str> = if base.is_empty() {
            Vec::new()
        } else {
            base.split('/').collect()
        };
        for _ in 0..dots.saturating_sub(1) {
            if parts.pop().is_none() {
                return;
            }
        }
        parts.extend(segs.iter().copied());
        let joined = parts.join("/");
        if let Some(hit) = python_file_for(&joined, paths) {
            internal.push(hit);
        }
        return;
    }

    // Absolute: resolve from the project root.
    let joined = segs.join("/");
    if let Some(hit) = python_file_for(&joined, paths) {
        internal.push(hit);
    } else if let Some(top) = segs.first() {
        external.push((*top).to_string());
    }
}

/// A `.py` file (or package `__init__.py`) for a `/`-joined module path.
fn python_file_for(joined: &str, paths: &HashSet<String>) -> Option<String> {
    if joined.is_empty() {
        return None;
    }
    let file = format!("{joined}.py");
    if paths.contains(&file) {
        return Some(file);
    }
    let init = format!("{joined}/__init__.py");
    if paths.contains(&init) {
        return Some(init);
    }
    None
}

// --- Rust -------------------------------------------------------------------

fn scan_rust(
    text: &str,
    importer: &str,
    paths: &HashSet<String>,
    internal: &mut Vec<String>,
    external: &mut Vec<String>,
) {
    let module_dir = rust_module_dir(importer);
    for line in text.lines() {
        let t = line.trim();
        // `mod name;` — a child module resolved to a real file.
        if let Some(rest) = t.strip_prefix("mod ") {
            if let Some(name) = rest.strip_suffix(';') {
                let name = name.trim();
                if is_ident(name) {
                    if let Some(hit) = rust_mod_file(&module_dir, name, paths) {
                        internal.push(hit);
                    }
                }
            }
            continue;
        }
        // `pub mod name;`
        if let Some(rest) = t.strip_prefix("pub mod ") {
            if let Some(name) = rest.strip_suffix(';') {
                let name = name.trim();
                if is_ident(name) {
                    if let Some(hit) = rust_mod_file(&module_dir, name, paths) {
                        internal.push(hit);
                    }
                }
            }
            continue;
        }
        // `use <crate>::...` — record the external crate root (skip the
        // in-crate roots crate/self/super, whose files the `mod` edges cover).
        if let Some(rest) = t
            .strip_prefix("use ")
            .or_else(|| t.strip_prefix("pub use "))
        {
            let head = rest
                .trim_start()
                .split([':', ';', ' ', '{', '<'])
                .next()
                .unwrap_or("");
            if is_ident(head) && !matches!(head, "crate" | "self" | "super") {
                external.push(head.to_string());
            }
        }
    }
}

/// The directory a file's `mod name;` declarations resolve against. For a
/// crate/module root (`lib.rs`, `main.rs`, `mod.rs`) that's the file's own
/// directory; otherwise (Rust 2018) it's a directory named after the file.
fn rust_module_dir(importer: &str) -> String {
    let dir = parent_dir(importer);
    let stem = importer
        .rsplit_once('/')
        .map(|(_, f)| f)
        .unwrap_or(importer)
        .strip_suffix(".rs")
        .unwrap_or("");
    if matches!(stem, "lib" | "main" | "mod") {
        dir.to_string()
    } else if dir.is_empty() {
        stem.to_string()
    } else {
        format!("{dir}/{stem}")
    }
}

fn rust_mod_file(module_dir: &str, name: &str, paths: &HashSet<String>) -> Option<String> {
    let prefix = if module_dir.is_empty() {
        String::new()
    } else {
        format!("{module_dir}/")
    };
    let flat = format!("{prefix}{name}.rs");
    if paths.contains(&flat) {
        return Some(flat);
    }
    let nested = format!("{prefix}{name}/mod.rs");
    if paths.contains(&nested) {
        return Some(nested);
    }
    None
}

fn is_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !s.chars().next().unwrap().is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(paths: &[&str]) -> HashSet<String> {
        paths.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn ts_relative_import_resolves_with_extension() {
        let paths = set(&["src/auth/login.ts", "src/auth/user.ts", "src/db/conn.ts"]);
        let text = "import { User } from './user';\nimport { db } from '../db/conn';\n";
        let r = scan(FileLang::TypeScript, text, "src/auth/login.ts", &paths);
        assert_eq!(r.internal, vec!["src/auth/user.ts", "src/db/conn.ts"]);
        assert!(r.external.is_empty());
    }

    #[test]
    fn ts_index_and_bare_imports() {
        let paths = set(&["src/api/index.ts", "src/app.ts"]);
        let text = "import { router } from './api';\nimport React from 'react';\nimport x from '@scope/pkg/sub';\n";
        let r = scan(FileLang::TypeScript, text, "src/app.ts", &paths);
        assert_eq!(r.internal, vec!["src/api/index.ts"]);
        assert_eq!(r.external, vec!["@scope/pkg", "react"]);
    }

    #[test]
    fn ts_require_form() {
        let paths = set(&["lib/util.js", "index.js"]);
        let text = "const util = require('./lib/util');\n";
        let r = scan(FileLang::JavaScript, text, "index.js", &paths);
        assert_eq!(r.internal, vec!["lib/util.js"]);
    }

    #[test]
    fn python_absolute_and_relative() {
        let paths = set(&[
            "app/api/routes.py",
            "app/db/__init__.py",
            "app/auth/login.py",
        ]);
        let text = "from app.db import session\nfrom .login import authenticate\nimport os\n";
        let r = scan(FileLang::Python, text, "app/auth/handler.py", &paths);
        assert_eq!(r.internal, vec!["app/auth/login.py", "app/db/__init__.py"]);
        assert_eq!(r.external, vec!["os"]);
    }

    #[test]
    fn python_parent_relative() {
        let paths = set(&["pkg/util.py", "pkg/sub/mod.py"]);
        let text = "from ..util import helper\n";
        let r = scan(FileLang::Python, text, "pkg/sub/mod.py", &paths);
        assert_eq!(r.internal, vec!["pkg/util.py"]);
    }

    #[test]
    fn rust_mod_flat_and_nested() {
        let paths = set(&["src/lib.rs", "src/model.rs", "src/build/mod.rs"]);
        let text = "mod model;\npub mod build;\nuse serde::Serialize;\nuse crate::model::Thing;\n";
        let r = scan(FileLang::Rust, text, "src/lib.rs", &paths);
        assert_eq!(r.internal, vec!["src/build/mod.rs", "src/model.rs"]);
        // `serde` is external; `crate` is skipped (covered by mod edges).
        assert_eq!(r.external, vec!["serde"]);
    }

    #[test]
    fn rust_2018_child_of_non_root() {
        let paths = set(&["src/commands.rs", "src/commands/scan.rs"]);
        let text = "mod scan;\n";
        let r = scan(FileLang::Rust, text, "src/commands.rs", &paths);
        assert_eq!(r.internal, vec!["src/commands/scan.rs"]);
    }

    #[test]
    fn unresolved_relative_is_dropped_not_invented() {
        let paths = set(&["src/app.ts"]);
        let text = "import x from './does-not-exist';\n";
        let r = scan(FileLang::TypeScript, text, "src/app.ts", &paths);
        assert!(r.internal.is_empty());
        assert!(r.external.is_empty());
    }

    #[test]
    fn go_records_nothing_internal() {
        let paths = set(&["main.go", "util.go"]);
        let text = "import \"fmt\"\nimport \"github.com/x/y\"\n";
        let r = scan(FileLang::Go, text, "main.go", &paths);
        assert!(r.internal.is_empty());
    }
}
