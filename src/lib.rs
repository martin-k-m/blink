//! Workspace root crate. Carries no runtime code of its own; it exists so
//! that `tests/` at the repository root can hold real Cargo integration
//! tests that exercise Blink's crates together (detection fixtures, and
//! black-box tests of the `blink` binary via `assert_cmd`).
