//! `.env` / `.env.example` comparison. Blink only ever reports variable
//! **names** — never values — so it's safe to run and safe to show.

use std::path::Path;

/// The result of comparing a project's `.env` against its `.env.example`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvReport {
    pub has_example: bool,
    pub has_env: bool,
    /// Declared in `.env.example` and present in `.env`.
    pub configured: Vec<String>,
    /// Declared in `.env.example` but absent from `.env`.
    pub missing: Vec<String>,
    /// Present in `.env` but not declared in `.env.example`.
    pub unused: Vec<String>,
}

impl EnvReport {
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty()
    }
}

/// Compare `.env` and `.env.example` in `root`. Missing files are treated as
/// "no variables," so a project with only one of the two still reports sensibly.
pub fn compare(root: &Path) -> EnvReport {
    let example_path = root.join(".env.example");
    let env_path = root.join(".env");

    let has_example = example_path.is_file();
    let has_env = env_path.is_file();

    let example = read_var_names(&example_path);
    let actual = read_var_names(&env_path);

    let configured: Vec<String> = example
        .iter()
        .filter(|k| actual.contains(*k))
        .cloned()
        .collect();
    let missing: Vec<String> = example
        .iter()
        .filter(|k| !actual.contains(*k))
        .cloned()
        .collect();
    let unused: Vec<String> = actual
        .iter()
        .filter(|k| !example.contains(*k))
        .cloned()
        .collect();

    EnvReport {
        has_example,
        has_env,
        configured,
        missing,
        unused,
    }
}

/// Extract the variable names from a `KEY=value` dotenv file, in file order,
/// deduplicated. Values are read only far enough to find the `=` and are never
/// retained. `export KEY=...` and comments/blank lines are handled.
pub fn read_var_names(path: &Path) -> Vec<String> {
    let raw = match std::fs::read_to_string(path) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((key, _)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty()
            || !key
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
        {
            continue;
        }
        if !out.contains(&key.to_string()) {
            out.push(key.to_string());
        }
    }
    out
}
