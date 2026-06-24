/// Returns the **resource pack** format number for a given Minecraft version
/// string.
///
/// Delegates to [`sand_core::version::VersionProfile`] so there is a single
/// source of truth for resource-pack format numbers.  For versions not in the
/// known table the latest known format is returned as a conservative fallback
/// — users can always override `resource_pack_format` in their `sand.toml`.
///
/// Prefer using `VersionProfile::resourcepack_metadata()` directly when you
/// need to detect fallback behaviour (e.g. to warn the user).
///
/// Reference: <https://minecraft.wiki/w/Resource_pack#Pack_format>
pub fn resource_pack_format_for(mc_version: &str) -> u32 {
    use sand_core::version::{MinecraftVersion, VersionProfile};
    MinecraftVersion::parse(mc_version)
        .ok()
        .and_then(|v| VersionProfile::resolve(&v).ok())
        .map(|p| p.resource_pack_format)
        .unwrap_or(61)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_versions() {
        assert_eq!(resource_pack_format_for("1.21.4"), 46);
        assert_eq!(resource_pack_format_for("1.21.5"), 57);
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
        assert_eq!(resource_pack_format_for("1.21.6"), 61);
        assert_eq!(resource_pack_format_for("1.22.0"), 61);
        assert_eq!(resource_pack_format_for("0.0.0"), 61);
    }
}
