//! Blink's workflow engine: rule-based project optimization, environment
//! diagnostics, task discovery, cleanup planning, storage analysis, `.env`
//! validation, duplicate detection, and configuration auditing.
//!
//! Everything here is fact-driven — every finding names the concrete condition
//! that produced it, and nothing claims a speedup it did not measure. The heavy
//! per-file work (hashes, symbols) is reused from [`blink_index`] rather than
//! recomputed.

pub mod clean;
pub mod config_audit;
pub mod doctor;
pub mod duplicates;
pub mod env;
pub mod filesystem;
mod fs_util;
pub mod git;
pub mod optimize;
pub mod tasks;

#[cfg(test)]
mod tests;

pub use clean::{plan as plan_clean, CleanTarget};
pub use config_audit::{audit as audit_config, AuditItem};
pub use doctor::{diagnose, Check, CheckStatus, DoctorReport};
pub use duplicates::{find as find_duplicates, DuplicateGroup};
pub use env::{compare as compare_env, EnvReport};
pub use filesystem::{analyze as analyze_filesystem, DirUsage, FilesystemReport};
pub use fs_util::{dir_size, on_path};
pub use optimize::{optimize, OptCheck, OptStatus, OptimizationReport};
pub use tasks::{discover as discover_tasks, Task, TaskSource};
