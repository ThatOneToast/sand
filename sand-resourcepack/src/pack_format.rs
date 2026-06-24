/// Returns the **resource pack** format number for a given Minecraft version
/// string.
///
/// Resource pack format numbers are a *separate* series from data pack format
/// numbers. For versions newer than the highest known entry the most recent
/// known value is used as a conservative fallback — users can always override
/// `resource_pack_format` in their `sand.toml`.
///
/// The canonical source of truth for these values is
/// `sand_core::version::VersionProfile::resource_pack_format`. This table
/// must remain consistent with that one.
///
/// Reference: <https://minecraft.wiki/w/Pack_format>
///
/// | Minecraft version | Resource pack format |
/// |---|---|
/// | 26.2.x | 88 |
/// | 26.1.x | 84 |
/// | 1.21.11 | 75 |
/// | 1.21.9–1.21.10 | 69 |
/// | 1.21.7–1.21.8 | 64 |
/// | 1.21.6 | 63 |
/// | 1.21.5 | 55 |
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
        // 26.x calendar series
        (26, 2, _) => 88,
        (26, 1, _) => 84,
        (26, _, _) => 88, // unknown 26.x: use latest known (26.2)
        // 1.21.x
        (1, 21, 11) => 75,
        (1, 21, 9..=10) => 69,
        (1, 21, 7..=8) => 64,
        (1, 21, 6) => 63,
        (1, 21, 5) => 55,
        (1, 21, 4) => 46,
        (1, 21, 2..=3) => 42,
        (1, 21, 0..=1) => 34,
        (1, 21, _) => 75, // unknown future 1.21.x: use latest known (1.21.11)
        // 1.20.x
        (1, 20, 5..=6) => 32,
        (1, 20, 3..=4) => 22,
        (1, 20, 2) => 18,
        (1, 20, 0..=1) => 15,
        // 1.19.x
        (1, 19, 4) => 13,
        (1, 19, 0..=3) => 12,
        // 1.18.x
        (1, 18, 0..=2) => 8,
        // anything else: conservative latest
        _ => 88,
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
        assert_eq!(resource_pack_format_for("1.21.5"), 55);
        assert_eq!(resource_pack_format_for("1.21.6"), 63);
        assert_eq!(resource_pack_format_for("1.21.7"), 64);
        assert_eq!(resource_pack_format_for("1.21.9"), 69);
        assert_eq!(resource_pack_format_for("1.21.11"), 75);
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
    fn known_26_series() {
        assert_eq!(resource_pack_format_for("26.1"), 84);
        assert_eq!(resource_pack_format_for("26.1.2"), 84);
        assert_eq!(resource_pack_format_for("26.2"), 88);
    }

    #[test]
    fn future_and_unknown_return_latest() {
        // Unknown versions return the latest known resource pack format (26.2 = 88).
        assert_eq!(resource_pack_format_for("26.3"), 88);
        assert_eq!(resource_pack_format_for("1.22.0"), 88);
        assert_eq!(resource_pack_format_for("0.0.0"), 88);
    }
}
