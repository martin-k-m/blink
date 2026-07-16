//! Duplicate-file detection built on the index's content hashes. Two files are
//! duplicates when their SHA-256 matches; grouping is exact, not fuzzy.

use std::collections::BTreeMap;

use blink_index::Index;

/// A set of two or more files with identical contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateGroup {
    pub hash: String,
    /// Size of one copy, in bytes.
    pub bytes: u64,
    /// Paths sharing this content, sorted.
    pub paths: Vec<String>,
}

impl DuplicateGroup {
    /// Space that could be reclaimed by keeping a single copy.
    pub fn wasted_bytes(&self) -> u64 {
        self.bytes * (self.paths.len() as u64 - 1)
    }
}

/// Find duplicate-content file groups in `index`, largest wasted space first.
/// Zero-byte files are ignored — empty files trivially collide and reclaiming
/// nothing isn't worth reporting.
pub fn find(index: &Index) -> Vec<DuplicateGroup> {
    let mut by_hash: BTreeMap<&str, Vec<&blink_index::FileRecord>> = BTreeMap::new();
    for record in index.files.values() {
        if record.size == 0 {
            continue;
        }
        by_hash
            .entry(record.hash.as_str())
            .or_default()
            .push(record);
    }

    let mut groups: Vec<DuplicateGroup> = by_hash
        .into_iter()
        .filter(|(_, recs)| recs.len() > 1)
        .map(|(hash, recs)| {
            let mut paths: Vec<String> = recs.iter().map(|r| r.path.clone()).collect();
            paths.sort();
            DuplicateGroup {
                hash: hash.to_string(),
                bytes: recs[0].size,
                paths,
            }
        })
        .collect();

    groups.sort_by(|a, b| {
        b.wasted_bytes()
            .cmp(&a.wasted_bytes())
            .then(a.paths[0].cmp(&b.paths[0]))
    });
    groups
}

/// Total reclaimable space across all duplicate groups.
pub fn total_wasted(groups: &[DuplicateGroup]) -> u64 {
    groups.iter().map(|g| g.wasted_bytes()).sum()
}
