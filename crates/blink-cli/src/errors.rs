//! Terminal rendering for command failures.
//!
//! Every Blink command returns `anyhow::Result`. When one fails, this module
//! turns the error into a `⚡`-branded block that (where Blink recognizes the
//! cause) explains *what* went wrong and offers concrete *fixes*, instead of a
//! bare one-line message. Recognition works by walking the error chain for a
//! [`BlinkError`], so a low-level cause still produces good guidance even when
//! it's wrapped in higher-level `.context(...)`.

use blink_core::BlinkError;
use colored::Colorize;

/// Print a failure report for `err` to stderr.
pub fn render(err: &anyhow::Error) {
    let bolt = "\u{26a1}".truecolor(255, 138, 0).bold();

    match find_blink_error(err) {
        Some(blink) => {
            let advice = advise(blink);
            eprintln!();
            eprintln!("{bolt} {}", advice.headline.bold());
            if let Some(reason) = advice.reason {
                eprintln!();
                for line in wrap(&reason, 68) {
                    eprintln!("  {line}");
                }
            }
            if !advice.fixes.is_empty() {
                eprintln!();
                eprintln!("  {}", "Possible fixes:".dimmed());
                for fix in advice.fixes {
                    eprintln!("    {} {fix}", "\u{2192}".truecolor(255, 138, 0));
                }
            }
            eprintln!();
        }
        None => {
            // No recognized Blink cause — fall back to the full context chain,
            // still branded so failures look consistent.
            eprintln!("{bolt} {}", format!("{err:#}").bold());
        }
    }
}

/// The first [`BlinkError`] anywhere in the error's cause chain, if any.
fn find_blink_error(err: &anyhow::Error) -> Option<&BlinkError> {
    err.chain()
        .find_map(|cause| cause.downcast_ref::<BlinkError>())
}

struct Advice {
    headline: String,
    reason: Option<String>,
    fixes: Vec<String>,
}

fn advise(err: &BlinkError) -> Advice {
    match err {
        BlinkError::UnknownProject(path) => Advice {
            headline: format!("Blink couldn't find a project in {}", show(path)),
            reason: Some(
                "That directory has no Cargo.toml, package.json, \
                 requirements.txt, or pyproject.toml, so Blink can't tell what \
                 kind of project it is."
                    .to_string(),
            ),
            fixes: vec![
                "Run the command from your project's root directory".to_string(),
                "Point Blink at the right place: blink <command> path/to/project".to_string(),
                "Start a project first (cargo init, npm init, ...)".to_string(),
            ],
        },
        BlinkError::PathNotFound(path) => Advice {
            headline: format!("Blink couldn't find {}", show(path)),
            reason: Some("That path doesn't exist.".to_string()),
            fixes: vec![
                "Check the path for typos".to_string(),
                "Create it first, or pass a path that exists".to_string(),
            ],
        },
        BlinkError::NotADirectory(path) => Advice {
            headline: format!("{} is a file, not a directory", show(path)),
            reason: Some("Blink commands operate on a project directory.".to_string()),
            fixes: vec!["Pass the directory that contains your project".to_string()],
        },
        BlinkError::ConfigParse { path, source } => Advice {
            headline: format!("Blink couldn't parse {}", show(path)),
            reason: Some(format!("The config file has invalid TOML: {source}")),
            fixes: vec![
                "Fix the reported syntax error".to_string(),
                "Re-check the file against docs/configuration.md".to_string(),
            ],
        },
        BlinkError::ManifestParse { path, source } => Advice {
            headline: format!("Blink couldn't parse the manifest {}", show(path)),
            reason: Some(format!("The manifest is malformed: {source}")),
            fixes: vec![
                "Fix the manifest so your own package manager can read it".to_string(),
                "Then re-run the command".to_string(),
            ],
        },
        BlinkError::ConfigSerialize(source) => Advice {
            headline: "Blink couldn't write its configuration".to_string(),
            reason: Some(format!("Serializing the config to TOML failed: {source}")),
            fixes: vec!["This is likely a bug — please report it".to_string()],
        },
        BlinkError::Io { path, source } => Advice {
            headline: format!("Blink couldn't read {}", show(path)),
            reason: Some(format!("{source}")),
            fixes: vec!["Check the file exists and you have permission to read it".to_string()],
        },
    }
}

/// Render a path for display, trimming the noisy `\\?\` Windows verbatim prefix.
fn show(path: &std::path::Path) -> String {
    let s = path.display().to_string();
    s.strip_prefix(r"\\?\").unwrap_or(&s).to_string()
}

/// Wrap `text` to `width` columns on word boundaries, for a tidy reason block.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if !current.is_empty() && current.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_project_error_is_recognized_through_context() {
        // Simulate what a command does: wrap the core error in context.
        let core = BlinkError::UnknownProject(std::path::PathBuf::from("/tmp/x"));
        let wrapped = anyhow::Error::new(core).context("could not analyze /tmp/x");
        let found = find_blink_error(&wrapped).expect("BlinkError found in chain");
        let advice = advise(found);
        assert!(advice.headline.contains("couldn't find a project"));
        assert!(!advice.fixes.is_empty());
    }

    #[test]
    fn wrap_breaks_on_word_boundaries() {
        let out = wrap("one two three four five", 9);
        assert!(out.iter().all(|l| l.len() <= 9));
        assert_eq!(out.join(" "), "one two three four five");
    }

    #[test]
    fn show_trims_windows_verbatim_prefix() {
        assert_eq!(show(std::path::Path::new(r"\\?\C:\a")), r"C:\a");
    }
}
