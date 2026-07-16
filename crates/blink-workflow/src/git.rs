//! Thin, read-only wrapper over `git log`, used by `timeline`, `hotspots`, and
//! `inspect`. Everything here degrades gracefully: no git binary, or not a
//! repository, yields empty results rather than an error, because these are
//! enrichment features, not core functionality.

use std::path::Path;
use std::process::Command;

use crate::fs_util::on_path;

/// Whether `root` is a git working tree and `git` is available to query it.
pub fn available(root: &Path) -> bool {
    on_path("git") && root.join(".git").exists()
}

/// How many times each tracked file has changed across history, most-changed
/// first. Empty when git is unavailable.
pub fn churn(root: &Path) -> Vec<(String, usize)> {
    let out = match run(root, &["log", "--pretty=format:", "--name-only"]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for line in out.lines() {
        let path = line.trim();
        if path.is_empty() {
            continue;
        }
        *counts.entry(path.to_string()).or_insert(0) += 1;
    }

    let mut ranked: Vec<(String, usize)> = counts.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    ranked
}

/// Distinct files touched by recent commits, newest first, up to `limit`.
pub fn recent_files(root: &Path, limit: usize) -> Vec<String> {
    let out = match run(
        root,
        &["log", "-n", "150", "--pretty=format:", "--name-only"],
    ) {
        Some(o) => o,
        None => return Vec::new(),
    };

    let mut seen: Vec<String> = Vec::new();
    for line in out.lines() {
        let path = line.trim();
        if path.is_empty() {
            continue;
        }
        if !seen.iter().any(|p| p == path) {
            seen.push(path.to_string());
            if seen.len() >= limit {
                break;
            }
        }
    }
    seen
}

/// Total number of commits reachable from HEAD, if determinable.
pub fn commit_count(root: &Path) -> Option<usize> {
    run(root, &["rev-list", "--count", "HEAD"]).and_then(|o| o.trim().parse().ok())
}

/// Subject lines of the most recent commits, newest first, up to `limit`.
pub fn recent_commits(root: &Path, limit: usize) -> Vec<String> {
    let n = limit.to_string();
    match run(root, &["log", "-n", &n, "--pretty=format:%s"]) {
        Some(o) => o.lines().map(|l| l.to_string()).collect(),
        None => Vec::new(),
    }
}

/// Run `git args` in `root`, returning stdout on success (exit 0), else `None`.
fn run(root: &Path, args: &[&str]) -> Option<String> {
    if !on_path("git") {
        return None;
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}
