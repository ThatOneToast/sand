/// Returns the data pack format number for a given Minecraft version string.
///
/// The format is derived from the major/minor/patch tuple. For versions
/// newer than the highest known entry the most recent known value is used
/// as a conservative fallback — users can always override `pack_format` in
/// their `sand.toml`.
///
/// Reference: <https://minecraft.wiki/w/Data_pack#Pack_format>
pub fn pack_format_for(mc_version: &str) -> u32 {
    let (major, minor, patch) = parse_version(mc_version);
    match (major, minor, patch) {
        (1, 21, p) if p >= 4 => 61,
        (1, 21, 2..=3) => 57,
        (1, 21, 0..=1) => 48,
        (1, 20, 5..=6) => 41,
        (1, 20, 3..=4) => 26,
        (1, 20, 2) => 18,
        (1, 20, 0..=1) => 15,
        (1, 19, 4) => 12,
        (1, 19, 0..=3) => 10,
        (1, 18, 0..=2) => 9,
        _ if major > 1 || (major == 1 && minor > 21) => 61, // future versions
        _ => 61,                                            // unknown, use latest
    }
}

fn parse_version(s: &str) -> (u32, u32, u32) {
    let parts: Vec<&str> = s.split('.').collect();
    let get = |i: usize| parts.get(i).and_then(|p| p.parse().ok()).unwrap_or(0);
    (get(0), get(1), get(2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_versions() {
        assert_eq!(pack_format_for("1.21.4"), 61);
        assert_eq!(pack_format_for("1.21.3"), 57);
        assert_eq!(pack_format_for("1.21.1"), 48);
        assert_eq!(pack_format_for("1.21.0"), 48);
        assert_eq!(pack_format_for("1.20.6"), 41);
        assert_eq!(pack_format_for("1.20.4"), 26);
        assert_eq!(pack_format_for("1.20.1"), 15);
    }

    #[test]
    fn future_version_returns_latest() {
        assert_eq!(pack_format_for("1.21.11"), 61);
        assert_eq!(pack_format_for("1.22.0"), 61);
    }

    #[test]
    fn unknown_version_returns_latest() {
        assert_eq!(pack_format_for("0.0.0"), 61);
    }
}
