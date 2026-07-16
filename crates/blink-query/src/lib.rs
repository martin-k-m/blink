//! Structured, **local** search over a [`ContextGraph`].
//!
//! This is not AI and does no inference. It tokenizes a query, drops common
//! question/stop words, and ranks the project's own areas, files, symbols,
//! dependencies, and commands by how well their names match the remaining
//! terms. Every result points at a real node in the graph — `blink query`
//! finds *relationships and structure*, not just raw text, but it never
//! invents an answer that isn't grounded in the model.

use blink_context::ContextGraph;

/// A ranked match against a named node in the graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// The node's identifier (area path, file path, dependency/command name).
    pub name: String,
    /// A short, factual descriptor (e.g. `"14 files · 320 symbols"`).
    pub detail: String,
    pub score: i32,
}

/// A ranked symbol match, carrying its location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolMatch {
    pub name: String,
    pub kind: String,
    pub file: String,
    pub line: usize,
    pub score: i32,
}

/// The grouped results of a query.
#[derive(Debug, Clone, Default)]
pub struct QueryResults {
    pub query: String,
    /// The meaningful search terms left after dropping stop/question words.
    pub terms: Vec<String>,
    pub areas: Vec<Match>,
    pub files: Vec<Match>,
    pub symbols: Vec<SymbolMatch>,
    pub dependencies: Vec<Match>,
    pub commands: Vec<Match>,
}

impl QueryResults {
    /// Whether anything at all matched.
    pub fn is_empty(&self) -> bool {
        self.areas.is_empty()
            && self.files.is_empty()
            && self.symbols.is_empty()
            && self.dependencies.is_empty()
            && self.commands.is_empty()
    }

    pub fn total(&self) -> usize {
        self.areas.len()
            + self.files.len()
            + self.symbols.len()
            + self.dependencies.len()
            + self.commands.len()
    }
}

/// Common words carrying no search signal — dropped so "where are the API
/// routes" searches for `api` + `routes`.
const STOP_WORDS: &[&str] = &[
    "a", "an", "the", "of", "in", "on", "at", "to", "for", "and", "or", "with", "that", "this",
    "is", "are", "was", "were", "be", "where", "what", "which", "who", "how", "do", "does", "did",
    "find", "show", "me", "my", "i", "get", "list", "all", "any", "it", "its", "from", "into",
    "there",
];

/// Run `query` against `graph`, returning up to `limit` results per group.
pub fn query(graph: &ContextGraph, raw: &str, limit: usize) -> QueryResults {
    let mut terms = meaningful_terms(raw);
    if terms.is_empty() {
        // Every token was a stop word (e.g. a query of just "where is"): fall
        // back to the raw tokens so the user still gets a best-effort answer.
        terms = tokenize(raw);
    }

    let mut results = QueryResults {
        query: raw.trim().to_string(),
        terms: terms.clone(),
        ..Default::default()
    };
    if terms.is_empty() {
        return results;
    }

    // Areas.
    for area in &graph.areas {
        let score = score(&area.path, &terms);
        if score > 0 {
            results.areas.push(Match {
                name: area.path.clone(),
                detail: format!("{} files · {} symbols", area.files, area.symbols),
                score,
            });
        }
    }

    // Files (match on path).
    for file in &graph.files {
        let score = score(&file.path, &terms);
        if score > 0 {
            let detail = match &file.lang {
                Some(lang) => format!(
                    "{lang} · {} in {}",
                    pluralize(file.symbols.len(), "symbol"),
                    file.area
                ),
                None => file.area.clone(),
            };
            results.files.push(Match {
                name: file.path.clone(),
                detail,
                score,
            });
        }
    }

    // Symbols.
    for file in &graph.files {
        for sym in &file.symbols {
            let score = score(&sym.name, &terms);
            if score > 0 {
                results.symbols.push(SymbolMatch {
                    name: sym.name.clone(),
                    kind: sym.kind.clone(),
                    file: file.path.clone(),
                    line: sym.line,
                    score,
                });
            }
        }
    }

    // Dependencies.
    for dep in &graph.dependencies {
        let score = score(&dep.name, &terms);
        if score > 0 {
            let detail = if dep.dev {
                format!("{} (dev)", dep.version)
            } else {
                dep.version.clone()
            };
            results.dependencies.push(Match {
                name: dep.name.clone(),
                detail,
                score,
            });
        }
    }

    // Commands.
    for cmd in &graph.commands {
        let score = score(&cmd.name, &terms).max(score(&cmd.command, &terms));
        if score > 0 {
            results.commands.push(Match {
                name: cmd.name.clone(),
                detail: cmd.command.clone(),
                score,
            });
        }
    }

    rank(&mut results.areas, limit);
    rank(&mut results.files, limit);
    rank_symbols(&mut results.symbols, limit);
    rank(&mut results.dependencies, limit);
    rank(&mut results.commands, limit);
    results
}

fn rank(matches: &mut Vec<Match>, limit: usize) {
    matches.sort_by(|a, b| b.score.cmp(&a.score).then(a.name.cmp(&b.name)));
    matches.truncate(limit);
}

fn rank_symbols(matches: &mut Vec<SymbolMatch>, limit: usize) {
    matches.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(a.name.cmp(&b.name))
            .then(a.file.cmp(&b.file))
    });
    matches.truncate(limit);
}

/// The query's tokens with stop words removed.
fn meaningful_terms(raw: &str) -> Vec<String> {
    tokenize(raw)
        .into_iter()
        .filter(|t| !STOP_WORDS.contains(&t.as_str()))
        .collect()
}

/// Lowercase alphanumeric tokens, splitting on punctuation and `camelCase`
/// boundaries.
fn tokenize(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for chunk in s.split(|c: char| !c.is_alphanumeric()) {
        if chunk.is_empty() {
            continue;
        }
        for tok in split_camel(chunk) {
            let lower = tok.to_ascii_lowercase();
            if !lower.is_empty() && !out.contains(&lower) {
                out.push(lower);
            }
        }
    }
    out
}

/// Split `camelCase`/`PascalCase`/`snake` identifiers into their word parts,
/// while keeping the whole token too (so `getUser` yields `getuser`, `get`,
/// `user`).
fn split_camel(s: &str) -> Vec<String> {
    let mut parts: Vec<String> = vec![s.to_string()];
    let mut current = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        let boundary = i > 0
            && c.is_uppercase()
            && (chars[i - 1].is_lowercase() || chars[i - 1].is_ascii_digit());
        if boundary && !current.is_empty() {
            parts.push(std::mem::take(&mut current));
        }
        current.push(c);
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

/// Score how well a candidate name matches the query terms. Zero means no
/// match (the candidate is dropped).
fn score(candidate: &str, terms: &[String]) -> i32 {
    let lower = candidate.to_ascii_lowercase();
    let toks = tokenize(candidate);
    let mut total = 0;
    let mut matched = 0;
    for term in terms {
        let mut best = 0;
        if toks.iter().any(|t| t == term) {
            best = 50; // whole-word/segment match
        } else if toks.iter().any(|t| t.starts_with(term.as_str())) {
            best = 25; // the term is a prefix of a word (`route` → `router`)
        } else if toks
            .iter()
            .any(|t| t.len() >= 3 && term.starts_with(t.as_str()))
        {
            best = 15; // a word is a prefix of the term (`connect` → `connection`)
        } else if lower.contains(term.as_str()) {
            best = 10; // substring somewhere
        }
        if best > 0 {
            matched += 1;
            total += best;
        }
    }
    if matched == 0 {
        return 0;
    }
    // Reward matching more of the query.
    total + (matched - 1) * 15
}

fn pluralize(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

#[cfg(test)]
mod tests;
