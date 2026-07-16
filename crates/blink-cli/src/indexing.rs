use std::path::Path;

use anyhow::{Context, Result};
use blink_core::BlinkConfig;
use blink_index::Index;

/// Get an up-to-date index for `root`, honoring `[index]` config.
///
/// - `[index].enabled = false` builds a throwaway in-memory index (nothing is
///   written to `.blink/`).
/// - otherwise the on-disk index is incrementally refreshed; when
///   `[index].auto_update` is true (the default) the refreshed index is saved
///   so the next command starts warm.
pub fn ensure_index(root: &Path) -> Result<Index> {
    let config = BlinkConfig::load(root).ok();
    let index_cfg = config.map(|c| c.index).unwrap_or_default();

    if !index_cfg.enabled {
        let (index, _) = Index::build(root).context("failed to build in-memory index")?;
        return Ok(index);
    }

    let (index, _stats) = Index::refresh(root).context("failed to refresh index")?;
    if index_cfg.auto_update {
        // Best-effort persistence: a read-only filesystem shouldn't fail a query.
        let _ = index.save();
    }
    Ok(index)
}
