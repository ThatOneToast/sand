/// Returns the data pack format number for a given Minecraft version string.
///
/// Delegates to [`sand_core::version::VersionProfile`] so there is a single
/// source of truth for pack-format numbers. Versions before 26 are rejected;
/// unknown future calendar versions use the conservative profile.
///
/// Reference: <https://minecraft.wiki/w/Data_pack#Pack_format>
pub fn pack_format_for(mc_version: &str) -> anyhow::Result<u32> {
    use sand_core::version::{MinecraftVersion, VersionProfile};
    let version = MinecraftVersion::parse(mc_version)?;
    Ok(VersionProfile::resolve(&version)?.data_pack_format())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_formats_follow_the_26_plus_profile_boundary() {
        assert_eq!(pack_format_for("26.1").unwrap(), 101);
        assert_eq!(pack_format_for("26.2").unwrap(), 107);
        assert_eq!(pack_format_for("27.0").unwrap(), 107);
        assert!(pack_format_for("25.9").is_err());
    }
}
