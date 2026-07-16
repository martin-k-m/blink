/// Strip leading semver range operators (`^`, `~`, `=`, `>`, `<`) from a
/// manifest version requirement, leaving (best-effort) a plain version
/// string suitable for comparison or for querying a registry API.
pub fn clean_version(raw: &str) -> String {
    raw.trim_start_matches(['^', '~', '=', '>', '<', ' '])
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::clean_version;

    #[test]
    fn strips_range_operators() {
        assert_eq!(clean_version("^1.2.3"), "1.2.3");
        assert_eq!(clean_version("~1.2.3"), "1.2.3");
        assert_eq!(clean_version(">=1.2.3"), "1.2.3");
        assert_eq!(clean_version("1.2.3"), "1.2.3");
    }
}
