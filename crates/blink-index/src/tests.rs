use std::fs;
use std::path::Path;

use tempfile::TempDir;

use crate::{extract_symbols, FileLang, Index, SymbolKind};

fn write(dir: &Path, rel: &str, contents: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[test]
fn rust_symbols_extracted_with_kinds_and_lines() {
    let src = "\
pub fn alpha() {}
struct Beta;
pub enum Gamma { A, B }
trait Delta {}
pub type Epsilon = u32;
fn not_a_keyword_fnord() {}
let fnord = 3; // must not match `fn`
";
    let syms = extract_symbols(FileLang::Rust, src);
    let names: Vec<_> = syms.iter().map(|s| (s.name.as_str(), s.kind)).collect();
    assert_eq!(
        names,
        vec![
            ("alpha", SymbolKind::Function),
            ("Beta", SymbolKind::Struct),
            ("Gamma", SymbolKind::Enum),
            ("Delta", SymbolKind::Trait),
            ("Epsilon", SymbolKind::TypeAlias),
            ("not_a_keyword_fnord", SymbolKind::Function),
        ]
    );
    // `alpha` is on line 1.
    assert_eq!(syms[0].line, 1);
}

#[test]
fn typescript_and_python_and_go_symbols() {
    let ts = "\
export function handler() {}
export const doThing = (x) => x + 1;
class Widget {}
interface Shape {}
export enum Color { Red }
export type Id = string;
const notAFn = 42;
";
    let ts_syms: Vec<_> = extract_symbols(FileLang::TypeScript, ts)
        .into_iter()
        .map(|s| (s.name, s.kind))
        .collect();
    assert!(ts_syms.contains(&("handler".to_string(), SymbolKind::Function)));
    assert!(ts_syms.contains(&("doThing".to_string(), SymbolKind::Function)));
    assert!(ts_syms.contains(&("Widget".to_string(), SymbolKind::Class)));
    assert!(ts_syms.contains(&("Shape".to_string(), SymbolKind::Interface)));
    assert!(ts_syms.contains(&("Color".to_string(), SymbolKind::Enum)));
    assert!(ts_syms.contains(&("Id".to_string(), SymbolKind::TypeAlias)));
    // A plain value assignment is not a function.
    assert!(!ts_syms.iter().any(|(n, _)| n == "notAFn"));

    let py = "def compute():\n    pass\nclass Model:\n    pass\n";
    let py_syms: Vec<_> = extract_symbols(FileLang::Python, py)
        .into_iter()
        .map(|s| (s.name, s.kind))
        .collect();
    assert_eq!(
        py_syms,
        vec![
            ("compute".to_string(), SymbolKind::Function),
            ("Model".to_string(), SymbolKind::Class),
        ]
    );

    let go = "func Run() {}\nfunc (s *Server) Handle() {}\ntype User struct {}\ntype Reader interface {}\n";
    let go_syms: Vec<_> = extract_symbols(FileLang::Go, go)
        .into_iter()
        .map(|s| (s.name, s.kind))
        .collect();
    assert_eq!(
        go_syms,
        vec![
            ("Run".to_string(), SymbolKind::Function),
            ("Handle".to_string(), SymbolKind::Function),
            ("User".to_string(), SymbolKind::Struct),
            ("Reader".to_string(), SymbolKind::Interface),
        ]
    );
}

#[test]
fn build_indexes_files_and_symbols() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "src/main.rs", "fn main() {}\nstruct App;\n");
    write(dir.path(), "README.md", "# hello\n");
    write(dir.path(), "target/junk.rs", "fn ignored() {}"); // ignored dir

    let (index, stats) = Index::build(dir.path()).unwrap();

    // main.rs + README.md, but not target/junk.rs.
    assert_eq!(index.file_count(), 2);
    assert_eq!(stats.added, 2);
    assert_eq!(stats.unchanged, 0);
    assert_eq!(index.symbol_count(), 2); // main, App
    assert!(index.files.contains_key("src/main.rs"));
    assert!(!index.files.keys().any(|k| k.contains("target")));
}

#[test]
fn refresh_is_incremental() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "a.rs", "fn a() {}\n");
    write(dir.path(), "b.rs", "fn b() {}\n");

    let (index, stats) = Index::build(dir.path()).unwrap();
    assert_eq!(stats.added, 2);
    index.save().unwrap();

    // No changes: a refresh should reuse both records.
    let (_, stats) = Index::refresh(dir.path()).unwrap();
    assert_eq!(stats.unchanged, 2);
    assert_eq!(stats.reprocessed(), 0);
    assert!(!stats.changed());

    // Change one file, add one, remove one.
    write(dir.path(), "a.rs", "fn a() {}\nfn a2() {}\n");
    write(dir.path(), "c.rs", "fn c() {}\n");
    fs::remove_file(dir.path().join("b.rs")).unwrap();

    let (index, stats) = Index::refresh(dir.path()).unwrap();
    assert_eq!(stats.updated, 1, "a.rs changed");
    assert_eq!(stats.added, 1, "c.rs is new");
    assert_eq!(stats.removed, 1, "b.rs gone");
    assert_eq!(stats.unchanged, 0);
    assert_eq!(index.file_count(), 2); // a.rs, c.rs
    assert_eq!(index.files["a.rs"].symbols.len(), 2);
}

#[test]
fn persists_and_reloads() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "x.py", "def x():\n    pass\n");
    let (index, _) = Index::build(dir.path()).unwrap();
    index.save().unwrap();

    let loaded = Index::load(dir.path()).expect("index should load");
    assert_eq!(loaded.file_count(), 1);
    assert_eq!(loaded.root, dir.path()); // root restored from load location
    assert_eq!(loaded.search_symbols(Some("x")).len(), 1);
}

#[test]
fn search_and_language_breakdown() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "src/lib.rs", "fn helper() {}\n");
    write(dir.path(), "src/app.ts", "function main() {}\n");
    let (index, _) = Index::build(dir.path()).unwrap();

    assert_eq!(index.search_paths("app").len(), 1);
    assert_eq!(index.search_paths(".rs").len(), 1);
    assert_eq!(index.search_symbols(Some("main")).len(), 1);

    let langs = index.language_breakdown();
    assert_eq!(langs.get(&FileLang::Rust).unwrap().0, 1);
    assert_eq!(langs.get(&FileLang::TypeScript).unwrap().0, 1);
}
