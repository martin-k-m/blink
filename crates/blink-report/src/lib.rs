//! Formats Blink's analysis output: health scoring visuals, tables, and a
//! JSON export shape. This crate has no terminal/color concerns of its own
//! (see `blink-cli` for that) — everything here is plain text or a
//! `serde`-serializable struct, so it stays simple to test and to reuse in
//! non-terminal contexts (a future dashboard, say).

mod bar;
mod html;
mod issues;
mod json;
mod label;
mod markdown;
mod table;

pub use bar::health_bar;
pub use html::render_html;
pub use issues::{issues, Issue};
pub use json::{to_json_report, JsonDependencies, JsonHealth, JsonReport};
pub use label::project_type_label;
pub use markdown::render_markdown;
pub use table::{dependency_stats_table, largest_packages_table};
