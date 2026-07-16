use colored::Colorize;

/// Print the `⚡ <title>` header used at the top of every command's output.
pub fn banner(title: &str) {
    println!();
    println!(
        "{} {}",
        "\u{26a1}".truecolor(255, 138, 0).bold(),
        title.bold()
    );
}

/// Print a `✓ <label>` progress line.
pub fn step(label: impl AsRef<str>) {
    println!("  {} {}", "\u{2713}".green().bold(), label.as_ref());
}

/// Print an aligned `label   value` row.
pub fn field(label: &str, value: impl std::fmt::Display) {
    println!("  {} {}", format!("{label:<17}").dimmed(), value);
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
