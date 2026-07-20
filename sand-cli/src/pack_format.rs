/// Returns the data pack format number for a given Minecraft version string.
///
/// Delegates to [`sand_core::version::VersionProfile`] so there is a single
/// source of truth for pack-format numbers.  For versions not in the known
/// table the latest known format is returned as a conservative fallback.
///
/// Prefer using `VersionProfile::datapack_metadata()` directly when you need
/// to detect fallback behaviour (e.g. to warn the user).
///
/// Reference: <https://minecraft.wiki/w/Data_pack#Pack_format>
pub fn pack_format_for(mc_version: &str) -> u32 {
    use sand_core::version::{MinecraftVersion, VersionProfile};
    MinecraftVersion::parse(mc_version)
        .ok()
        .and_then(|v| VersionProfile::resolve(&v).ok())
        .map(|p| p.data_pack_format)
        .unwrap_or(107)
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
    fn known_recent_versions() {
        assert_eq!(pack_format_for("1.21.5"), 71);
        assert_eq!(pack_format_for("1.21.6"), 80);
        assert_eq!(pack_format_for("1.21.11"), 94);
        assert_eq!(pack_format_for("26.1"), 101);
        assert_eq!(pack_format_for("26.2"), 107);
    }

    #[test]
    fn unknown_version_returns_conservative_latest() {
        // Unknown versions return the conservative fallback (latest known: 26.2 = 107).
        assert_eq!(pack_format_for("1.22.0"), 107);
        assert_eq!(pack_format_for("0.0.0"), 107);
    }
}
