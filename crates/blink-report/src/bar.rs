/// Render a `score` (0-100) as a block-character progress bar with the
/// percentage appended, e.g. `█████████░ 92%`.
pub fn health_bar(score: u8, width: usize) -> String {
    let score = score.min(100);
    let filled = ((score as f32 / 100.0) * width as f32).round() as usize;
    let filled = filled.min(width);
    format!(
        "{}{} {score}%",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(width - filled)
    )
}

#[cfg(test)]
mod tests {
    use super::health_bar;

    #[test]
    fn renders_full_bar_at_100() {
        assert_eq!(health_bar(100, 10), "\u{2588}".repeat(10) + " 100%");
    }

    #[test]
    fn renders_empty_bar_at_zero() {
        assert_eq!(health_bar(0, 10), "\u{2591}".repeat(10) + " 0%");
    }

    #[test]
    fn rounds_to_nearest_segment() {
        // 92% of 10 segments = 9.2, rounds to 9 filled.
        assert_eq!(
            health_bar(92, 10),
            "\u{2588}".repeat(9) + "\u{2591}" + " 92%"
        );
    }

    #[test]
    fn clamps_scores_above_100() {
        assert_eq!(health_bar(150, 10), "\u{2588}".repeat(10) + " 100%");
    }
}
