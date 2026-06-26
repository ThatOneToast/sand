use sand_core::version::{LATEST_KNOWN, MinecraftVersion, VersionProfile};

/// Returns the resource pack format number for a given Minecraft version string.
///
/// This is a thin compatibility wrapper around [`VersionProfile::resourcepack_metadata`],
/// which is the single canonical source of truth for pack format numbers. For
/// unknown or future versions the most recent known value is returned as a
/// conservative fallback — users can always override `resource_pack_format` in
/// their `sand.toml`.
///
/// Reference: <https://minecraft.wiki/w/Pack_format>
pub fn resource_pack_format_for(mc_version: &str) -> u32 {
    resolve_resource_pack_format(mc_version).0
}

/// Resolve the resource pack format and whether it came from a conservative fallback.
///
/// Returns `(format, is_fallback)`. `is_fallback` is `true` when the exact
/// version was not found in the known table and a conservative default was used.
/// Callers that can warn the user (e.g. the build pipeline) should check this.
pub(crate) fn resolve_resource_pack_format(mc_version: &str) -> (u32, bool) {
    let v = match MinecraftVersion::parse(mc_version) {
        Ok(v) => v,
        Err(_) => {
            let latest = MinecraftVersion::parse(LATEST_KNOWN).unwrap();
            let profile = VersionProfile::resolve(&latest).unwrap();
            let meta = profile.resourcepack_metadata();
            return (meta.pack_format, true);
        }
    };

    let profile = VersionProfile::resolve(&v).unwrap_or_else(|_| {
        let latest = MinecraftVersion::parse(LATEST_KNOWN).unwrap();
        VersionProfile::resolve(&latest).unwrap()
    });
    let meta = profile.resourcepack_metadata();
    (meta.pack_format, meta.is_fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_version_26_2_resolves_to_88() {
        let (fmt, is_fallback) = resolve_resource_pack_format("26.2");
        assert_eq!(fmt, 88);
        assert!(!is_fallback, "26.2 is a known mapped version");
    }

    #[test]
    fn future_version_fallback() {
        let (fmt, is_fallback) = resolve_resource_pack_format("26.99");
        // Conservative fallback: uses latest known resource pack format (88 for 26.2)
        assert_eq!(
            fmt, 88,
            "unknown 26.99 should fall back to latest known (88)"
        );
        assert!(is_fallback, "26.99 is beyond the known table");
    }

    #[test]
    fn compatibility_wrapper_matches_version_profile() {
        // Verify that resource_pack_format_for delegates correctly for all known versions.
        let known_versions = [
            ("1.18.0", 8u32),
            ("1.18.2", 8),
            ("1.19.0", 12),
            ("1.19.4", 13),
            ("1.20.0", 15),
            ("1.20.2", 18),
            ("1.20.3", 22),
            ("1.20.5", 32),
            ("1.21.0", 34),
            ("1.21.2", 42),
            ("1.21.4", 46),
            ("1.21.5", 55),
            ("1.21.6", 63),
            ("1.21.7", 64),
            ("1.21.9", 69),
            ("1.21.11", 75),
            ("26.1", 84),
            ("26.2", 88),
        ];

        for (ver, expected) in known_versions {
            let from_wrapper = resource_pack_format_for(ver);
            assert_eq!(
                from_wrapper, expected,
                "resource_pack_format_for({ver}) returned {from_wrapper}, expected {expected}"
            );

            // Also confirm the wrapper agrees with VersionProfile directly.
            let v = MinecraftVersion::parse(ver).unwrap();
            let p = VersionProfile::resolve(&v).unwrap();
            let from_profile = p.resourcepack_metadata().pack_format;
            assert_eq!(
                from_wrapper, from_profile,
                "wrapper and VersionProfile diverged for {ver}: wrapper={from_wrapper}, profile={from_profile}"
            );
        }
    }

    #[test]
    fn no_duplicate_table_regression() {
        // Confirm that resolve_resource_pack_format always agrees with
        // VersionProfile::resourcepack_metadata() for known versions.
        // This test fails if a second independent table is ever introduced.
        let spot_checks = ["1.21.4", "1.21.11", "26.1", "26.2"];
        for ver in spot_checks {
            let (wrapper_fmt, _) = resolve_resource_pack_format(ver);
            let v = MinecraftVersion::parse(ver).unwrap();
            let profile_fmt = VersionProfile::resolve(&v)
                .unwrap()
                .resourcepack_metadata()
                .pack_format;
            assert_eq!(
                wrapper_fmt, profile_fmt,
                "format mismatch for {ver}: resolve_resource_pack_format={wrapper_fmt}, VersionProfile={profile_fmt}"
            );
        }
    }
}
