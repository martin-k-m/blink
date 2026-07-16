//! Lightweight, dependency-free symbol extraction.
//!
//! This is intentionally a line-oriented scanner, not a real parser: it finds
//! top-level `fn`/`struct`/`class`/… declarations by keyword and pulls out the
//! following identifier. It does not understand scope, macros, or generics
//! beyond stopping at the first delimiter, and it deliberately errs toward
//! *missing* an oddly-formatted declaration rather than inventing one — in
//! keeping with Blink's "no fabricated output" rule. It is accurate enough to
//! power `blink symbols`/`search` over ordinary code and is covered by tests
//! against real snippets.

use crate::record::{FileLang, Symbol, SymbolKind};

/// Extract top-level symbols from `text` written in `lang`.
pub fn extract_symbols(lang: FileLang, text: &str) -> Vec<Symbol> {
    let mut out = Vec::new();
    for (idx, raw) in text.lines().enumerate() {
        let line = strip_line(raw);
        if line.is_empty() {
            continue;
        }
        let found = match lang {
            FileLang::Rust => rust_symbol(line),
            FileLang::Python => python_symbol(line),
            FileLang::TypeScript => ts_symbol(line, true),
            FileLang::JavaScript => ts_symbol(line, false),
            FileLang::Go => go_symbol(line),
        };
        if let Some((name, kind)) = found {
            out.push(Symbol {
                name,
                kind,
                line: idx + 1,
            });
        }
    }
    out
}

/// Trim whitespace and drop common leading visibility/modifier keywords so the
/// declaration keyword is first. Also skips full-line comments.
fn strip_line(raw: &str) -> &str {
    let mut line = raw.trim_start();
    // Skip line comments and doc-comment continuation lines. `#` is left alone
    // because it's a Rust attribute *and* a Python comment; the per-language
    // matchers below simply won't find a declaration keyword on those lines.
    if line.starts_with("//") || line.starts_with('*') {
        return "";
    }
    for prefix in [
        "pub(crate) ",
        "pub(super) ",
        "pub ",
        "export default ",
        "export ",
        "default ",
        "async ",
        "unsafe ",
        "const ",
        "static ",
    ] {
        // `const`/`static` are stripped only as Rust/JS modifiers when another
        // keyword follows; the JS arrow-const case is handled explicitly in
        // `ts_symbol`, so don't strip a bare `const NAME = ...` here.
        if (prefix == "const " || prefix == "static ") && line.starts_with(prefix) {
            continue;
        }
        if let Some(rest) = line.strip_prefix(prefix) {
            line = rest.trim_start();
        }
    }
    line.trim()
}

/// Read the identifier at the start of `s` (letters, digits, `_`), stopping at
/// the first other character. Returns `None` if it isn't a valid identifier.
fn read_ident(s: &str) -> Option<String> {
    let s = s.trim_start();
    let mut chars = s.char_indices();
    let (_, first) = chars.next()?;
    if !(first.is_alphabetic() || first == '_') {
        return None;
    }
    let mut end = first.len_utf8();
    for (i, c) in chars {
        if c.is_alphanumeric() || c == '_' {
            end = i + c.len_utf8();
        } else {
            break;
        }
    }
    Some(s[..end].to_string())
}

fn after_keyword<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(keyword)?;
    // The keyword must be followed by whitespace, otherwise `fnord` matches `fn`.
    if rest.starts_with(|c: char| c.is_whitespace()) {
        Some(rest.trim_start())
    } else {
        None
    }
}

fn rust_symbol(line: &str) -> Option<(String, SymbolKind)> {
    let table = [
        ("fn ", SymbolKind::Function),
        ("struct ", SymbolKind::Struct),
        ("enum ", SymbolKind::Enum),
        ("trait ", SymbolKind::Trait),
        ("type ", SymbolKind::TypeAlias),
    ];
    for (kw, kind) in table {
        if let Some(rest) = after_keyword(line, kw.trim_end()) {
            if let Some(name) = read_ident(rest) {
                return Some((name, kind));
            }
        }
    }
    None
}

fn python_symbol(line: &str) -> Option<(String, SymbolKind)> {
    if let Some(rest) = after_keyword(line, "def") {
        return read_ident(rest).map(|n| (n, SymbolKind::Function));
    }
    if let Some(rest) = after_keyword(line, "class") {
        return read_ident(rest).map(|n| (n, SymbolKind::Class));
    }
    None
}

fn ts_symbol(line: &str, typescript: bool) -> Option<(String, SymbolKind)> {
    if let Some(rest) = after_keyword(line, "function") {
        // Skip a possible `*` for generators.
        let rest = rest.trim_start_matches('*').trim_start();
        return read_ident(rest).map(|n| (n, SymbolKind::Function));
    }
    if let Some(rest) = after_keyword(line, "class") {
        return read_ident(rest).map(|n| (n, SymbolKind::Class));
    }
    if typescript {
        if let Some(rest) = after_keyword(line, "interface") {
            return read_ident(rest).map(|n| (n, SymbolKind::Interface));
        }
        if let Some(rest) = after_keyword(line, "enum") {
            return read_ident(rest).map(|n| (n, SymbolKind::Enum));
        }
        if let Some(rest) = after_keyword(line, "type") {
            if let Some(name) = read_ident(rest) {
                // Require an `=` so `type` used as an identifier elsewhere is ignored.
                if rest[name.len()..].trim_start().starts_with('=') {
                    return Some((name, SymbolKind::TypeAlias));
                }
            }
        }
    }
    // `const NAME = (...) =>` / `const NAME = function` — an assigned function.
    for kw in ["const", "let", "var"] {
        if let Some(rest) = after_keyword(line, kw) {
            if let Some(name) = read_ident(rest) {
                let tail = rest[name.len()..].trim_start();
                if let Some(expr) = tail.strip_prefix('=') {
                    let expr = expr.trim_start();
                    if expr.contains("=>") || expr.starts_with("function") {
                        return Some((name, SymbolKind::Function));
                    }
                }
            }
        }
    }
    None
}

fn go_symbol(line: &str) -> Option<(String, SymbolKind)> {
    if let Some(rest) = after_keyword(line, "func") {
        // Method receiver: `func (r Recv) Name(...)`.
        let rest = if rest.starts_with('(') {
            rest.split_once(')').map(|(_, after)| after.trim_start())?
        } else {
            rest
        };
        return read_ident(rest).map(|n| (n, SymbolKind::Function));
    }
    if let Some(rest) = after_keyword(line, "type") {
        if let Some(name) = read_ident(rest) {
            let tail = rest[name.len()..].trim_start();
            if tail.starts_with("interface") {
                return Some((name, SymbolKind::Interface));
            }
            return Some((name, SymbolKind::Struct));
        }
    }
    None
}
