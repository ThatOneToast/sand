/// Returns the **resource pack** format number for a given Minecraft version
/// string.
///
/// Resource pack format numbers are a *separate* series from data pack format
/// numbers. For versions newer than the highest known entry the most recent
/// known value is used as a conservative fallback — users can always override
/// `resource_pack_format` in their `sand.toml`.
///
/// Reference: <https://minecraft.wiki/w/Resource_pack#Pack_format>
///
/// | Minecraft version | Resource pack format |
/// |---|---|
/// | 1.21.6+ | 61 |
/// | 1.21.5 | 57 |
/// | 1.21.4 | 46 |
/// | 1.21.2–1.21.3 | 42 |
/// | 1.21.0–1.21.1 | 34 |
/// | 1.20.5–1.20.6 | 32 |
/// | 1.20.3–1.20.4 | 22 |
/// | 1.20.2 | 18 |
/// | 1.20.0–1.20.1 | 15 |
/// | 1.19.4 | 13 |
/// | 1.19.0–1.19.3 | 12 |
/// | 1.18.0–1.18.2 | 8 |
pub fn resource_pack_format_for(mc_version: &str) -> u32 {
    let (major, minor, patch) = parse_version(mc_version);
    match (major, minor, patch) {
        (1, 21, p) if p >= 6 => 61,
        (1, 21, 5) => 57,
        (1, 21, 4) => 46,
        (1, 21, 2..=3) => 42,
        (1, 21, 0..=1) => 34,
        (1, 20, 5..=6) => 32,
        (1, 20, 3..=4) => 22,
        (1, 20, 2) => 18,
        (1, 20, 0..=1) => 15,
        (1, 19, 4) => 13,
        (1, 19, 0..=3) => 12,
        (1, 18, 0..=2) => 8,
        _ if major > 1 || (major == 1 && minor > 21) => 61, // future: use latest
        _ => 61,                                            // unknown: use latest
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
        assert_eq!(resource_pack_format_for("1.21.4"), 46);
        assert_eq!(resource_pack_format_for("1.21.3"), 42);
        assert_eq!(resource_pack_format_for("1.21.1"), 34);
        assert_eq!(resource_pack_format_for("1.21.0"), 34);
        assert_eq!(resource_pack_format_for("1.20.6"), 32);
        assert_eq!(resource_pack_format_for("1.20.4"), 22);
        assert_eq!(resource_pack_format_for("1.20.1"), 15);
        assert_eq!(resource_pack_format_for("1.19.4"), 13);
        assert_eq!(resource_pack_format_for("1.18.1"), 8);
    }

    #[test]
    fn future_and_unknown_return_latest() {
        assert_eq!(resource_pack_format_for("1.21.11"), 61);
        assert_eq!(resource_pack_format_for("1.22.0"), 61);
        assert_eq!(resource_pack_format_for("0.0.0"), 61);
    }
}
