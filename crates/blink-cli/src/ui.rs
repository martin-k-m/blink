use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

/// Print the `⚡ <title>` header used at the top of every command's output.
pub fn banner(title: &str) {
    println!();
    println!(
        "{} {}",
        "\u{26a1}".truecolor(255, 45, 141).bold(),
        title.bold()
    );
}

/// Print a `✓ <label>` progress line.
pub fn step(label: impl AsRef<str>) {
    println!("  {} {}", "\u{2713}".green().bold(), label.as_ref());
}

/// Print a `⚠ <label>` warning line.
pub fn warning(label: impl AsRef<str>) {
    println!("    {} {}", "\u{26a0}".yellow().bold(), label.as_ref());
}

/// Print a `→ <label>` suggestion line.
pub fn suggestion(label: impl AsRef<str>) {
    println!(
        "    {} {}",
        "\u{2192}".truecolor(255, 45, 141),
        label.as_ref()
    );
}

/// Print an aligned `label   value` row.
pub fn field(label: &str, value: impl std::fmt::Display) {
    println!("  {} {}", format!("{label:<17}").dimmed(), value);
}

/// A spinner for a step of unknown duration. Silent (drawn to nowhere) when
/// stderr isn't an interactive terminal, so piped output (CI logs, test
/// harnesses) never gets spinner control characters mixed into it.
pub fn spinner(message: &str) -> ProgressBar {
    if !console::Term::stderr().is_term() {
        return ProgressBar::hidden();
    }

    let bar = ProgressBar::new_spinner();
    bar.set_style(
        ProgressStyle::with_template("  {spinner:.yellow} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    bar.enable_steady_tick(std::time::Duration::from_millis(80));
    bar.set_message(message.to_string());
    bar
}

/// Print a trailing `label value` summary line, with a blank line above it.
pub fn footer(label: &str, value: impl std::fmt::Display) {
    println!();
    println!("  {} {}", label.dimmed(), value.to_string().bold());
}

/// Format a count with thousands separators, e.g. `2431` -> `2,431`.
pub fn format_count(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().rev().enumerate() {
        if i != 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::format_count;

    #[test]
    fn formats_thousands_separators() {
        assert_eq!(format_count(0), "0");
        assert_eq!(format_count(42), "42");
        assert_eq!(format_count(2431), "2,431");
        assert_eq!(format_count(1234567), "1,234,567");
    }
}
