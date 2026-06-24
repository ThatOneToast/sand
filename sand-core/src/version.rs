//! Minecraft version compatibility layer.
//!
//! Provides a single source of truth for version parsing, pack format lookup,
//! and feature flags across the 1.21.x and 26.x Java Edition series.
//!
//! # Quick start
//! ```
//! use sand_core::version::{MinecraftVersion, VersionProfile};
//!
//! let v = MinecraftVersion::parse("1.21.4").unwrap();
//! let profile = VersionProfile::resolve(&v).unwrap();
//! assert_eq!(profile.data_pack_format, 61);
//! assert_eq!(profile.resource_pack_format, 46);
//! assert!(profile.supports_item_components);
//! ```

use std::fmt;

use thiserror::Error;

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors from version parsing or profile resolution.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum VersionError {
    /// The version string could not be parsed.
    #[error(
        "Invalid version '{0}': expected '1.21', '1.21.4', '26', '26.1', '26.1.2', or 'latest'"
    )]
    ParseError(String),
    /// The version was parsed but is not in the known table.
    ///
    /// Use [`VersionProfile::resolve`] (which returns a conservative fallback) or
    /// add `pack_format` / `resource_pack_format` overrides to `sand.toml`.
    #[error("Unknown or unverified Minecraft version '{requested}'. {hint}")]
    UnknownVersion { requested: String, hint: String },
}

// ── MinecraftVersion ──────────────────────────────────────────────────────────

/// A parsed Minecraft Java Edition version.
///
/// Supports the legacy `1.x.y` series, the new `26.x` calendar series, and
/// the special `latest` token which resolves to the newest known entry.
///
/// # Examples
/// ```
/// use sand_core::version::MinecraftVersion;
///
/// let a = MinecraftVersion::parse("1.21.4").unwrap();
/// let b = MinecraftVersion::parse("26.1").unwrap();
/// let c = MinecraftVersion::parse("latest").unwrap();
/// assert!(a.is_121_series());
/// assert!(b.is_26_series());
/// assert!(c.is_latest());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinecraftVersion {
    kind: VersionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum VersionKind {
    Specific { major: u32, minor: u32, patch: u32 },
    Latest,
}

impl MinecraftVersion {
    /// Parse a version string into a `MinecraftVersion`.
    ///
    /// Accepted formats: `"1.21"`, `"1.21.4"`, `"26"`, `"26.1"`, `"26.1.2"`, `"latest"`.
    pub fn parse(s: &str) -> Result<Self, VersionError> {
        if s == "latest" {
            return Ok(Self {
                kind: VersionKind::Latest,
            });
        }
        let parts: Vec<&str> = s.split('.').collect();
        let parse_u32 = |p: &str| {
            p.parse::<u32>()
                .map_err(|_| VersionError::ParseError(s.to_string()))
        };
        let kind = match parts.as_slice() {
            [major] => VersionKind::Specific {
                major: parse_u32(major)?,
                minor: 0,
                patch: 0,
            },
            [major, minor] => VersionKind::Specific {
                major: parse_u32(major)?,
                minor: parse_u32(minor)?,
                patch: 0,
            },
            [major, minor, patch] => VersionKind::Specific {
                major: parse_u32(major)?,
                minor: parse_u32(minor)?,
                patch: parse_u32(patch)?,
            },
            _ => return Err(VersionError::ParseError(s.to_string())),
        };
        Ok(Self { kind })
    }

    /// Returns `true` if this is the `latest` token.
    pub fn is_latest(&self) -> bool {
        matches!(self.kind, VersionKind::Latest)
    }

    /// Returns `true` for the legacy `1.x` version series (e.g. `1.21.4`).
    pub fn is_121_series(&self) -> bool {
        matches!(self.kind, VersionKind::Specific { major: 1, .. })
    }

    /// Returns `true` for the new `26.x` calendar series.
    pub fn is_26_series(&self) -> bool {
        matches!(self.kind, VersionKind::Specific { major: 26, .. })
    }

    /// Return major, minor, patch components if this is a specific version.
    pub fn components(&self) -> Option<(u32, u32, u32)> {
        match self.kind {
            VersionKind::Specific {
                major,
                minor,
                patch,
            } => Some((major, minor, patch)),
            VersionKind::Latest => None,
        }
    }
}

impl fmt::Display for MinecraftVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            VersionKind::Latest => write!(f, "latest"),
            VersionKind::Specific {
                major,
                minor,
                patch,
            } => {
                write!(f, "{major}.{minor}.{patch}")
            }
        }
    }
}

// ── VersionProfile ────────────────────────────────────────────────────────────

/// Resolved compatibility profile for a Minecraft version.
///
/// The profile captures the pack format numbers and feature flags for the
/// requested version. For unknown or future versions a conservative fallback
/// is used — users can always override `pack_format` and
/// `resource_pack_format` in `sand.toml`.
///
/// # Examples
/// ```
/// use sand_core::version::{MinecraftVersion, VersionProfile};
///
/// let v = MinecraftVersion::parse("26.1").unwrap();
/// let p = VersionProfile::resolve(&v).unwrap();
/// assert!(p.supports_26_series);
/// assert!(p.supports_item_components);
/// ```
#[derive(Debug, Clone)]
pub struct VersionProfile {
    /// The version that was requested.
    pub requested: MinecraftVersion,
    /// Human-readable resolved name (e.g. `"1.21.4"` or `"26.1 (26-series fallback)"`).
    pub resolved_name: String,
    /// Data pack format number for `pack.mcmeta`.
    pub data_pack_format: u32,
    /// Resource pack format number for `pack.mcmeta`.
    pub resource_pack_format: u32,
    /// Whether this version supports item components (data components, 1.20.5+).
    pub supports_item_components: bool,
    /// Whether this version supports `data modify` components (1.20.2+).
    pub supports_data_components: bool,
    /// Whether this is the new 26.x calendar-versioned series.
    pub supports_26_series: bool,
    /// Whether this version supports data-driven dialogs (1.21.6+ / 26.x).
    pub supports_dialogs: bool,
    /// Whether this version supports function macros — `$()` syntax (1.20.2+).
    pub supports_function_macros: bool,
    /// Whether this version supports predicates (always true in 1.15+, our minimum).
    pub supports_predicates: bool,
    /// Whether this version supports resource pack overlays (1.20.2+).
    pub supports_resource_pack_overlays: bool,
    /// Whether this version supports trim assets — armor trims (1.19.4+).
    pub supports_trim_assets: bool,
    /// Whether this version supports jukebox song components (1.21+).
    pub supports_jukebox_songs: bool,
    /// Whether this version supports damage type registries (1.19.4+).
    pub supports_damage_types: bool,
    /// Whether this version supports chat type registries (1.19+).
    pub supports_chat_types: bool,
    /// Whether this version supports enchantment data components (1.21+).
    pub supports_enchantments: bool,
    /// When `true` the profile was resolved via a conservative fallback because
    /// the exact version was not in the known table. Users should verify and
    /// may override `pack_format` in `sand.toml`.
    pub is_fallback: bool,
}

/// The latest version this table was last verified against.
pub const LATEST_KNOWN: &str = "1.21.11";

// ── PackMetadata ──────────────────────────────────────────────────────────────

/// Resolved `pack.mcmeta` metadata for a single pack root.
///
/// Obtain via [`VersionProfile::datapack_metadata`] or
/// [`VersionProfile::resourcepack_metadata`].
///
/// # Example
/// ```
/// use sand_core::version::{MinecraftVersion, VersionProfile};
///
/// let v = MinecraftVersion::parse("1.21.4").unwrap();
/// let p = VersionProfile::resolve(&v).unwrap();
/// let meta = p.datapack_metadata();
/// assert_eq!(meta.pack_format, 61);
/// assert!(!meta.is_fallback);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackMetadata {
    /// The `pack.pack_format` value to write to `pack.mcmeta`.
    pub pack_format: u32,
    /// `true` if this metadata was resolved from a conservative fallback because
    /// the exact version was not in the known table.  The caller should warn
    /// the user and accept an override from `sand.toml`.
    pub is_fallback: bool,
}

impl VersionProfile {
    /// Resolve a [`MinecraftVersion`] into a [`VersionProfile`].
    ///
    /// Returns `Ok(profile)` for any parseable version. Unknown future versions
    /// receive a conservative fallback profile (see [`VersionProfile::is_fallback`]).
    pub fn resolve(version: &MinecraftVersion) -> Result<Self, VersionError> {
        let (major, minor, patch) = match version.components() {
            Some(c) => c,
            None => {
                // "latest" → use the newest known version
                let latest = MinecraftVersion::parse(LATEST_KNOWN).unwrap();
                let mut p = Self::resolve(&latest)?;
                p.requested = version.clone();
                p.resolved_name = format!("latest (resolved to {LATEST_KNOWN})");
                return Ok(p);
            }
        };

        let caps = lookup(major, minor, patch);
        let supports_26 = major >= 26;

        Ok(Self {
            requested: version.clone(),
            resolved_name: format!("{major}.{minor}.{patch}"),
            data_pack_format: caps.data_fmt,
            resource_pack_format: caps.res_fmt,
            supports_item_components: caps.item_components,
            supports_data_components: caps.data_components,
            supports_26_series: supports_26,
            supports_dialogs: caps.dialogs,
            supports_function_macros: caps.function_macros,
            supports_predicates: caps.predicates,
            supports_resource_pack_overlays: caps.resource_pack_overlays,
            supports_trim_assets: caps.trim_assets,
            supports_jukebox_songs: caps.jukebox_songs,
            supports_damage_types: caps.damage_types,
            supports_chat_types: caps.chat_types,
            supports_enchantments: caps.enchantments,
            is_fallback: caps.is_fallback,
        })
    }

    // ── Convenience predicate methods ─────────────────────────────────────────

    /// Returns `true` if data-driven dialogs are supported (1.21.6+ / 26.x).
    pub fn supports_dialogs(&self) -> bool {
        self.supports_dialogs
    }

    /// Returns `true` if function macros (`$()` syntax) are supported (1.20.2+).
    pub fn supports_function_macros(&self) -> bool {
        self.supports_function_macros
    }

    /// Returns `true` if resource pack overlays are supported (1.20.2+).
    pub fn supports_resource_pack_overlays(&self) -> bool {
        self.supports_resource_pack_overlays
    }

    /// Returns `true` if jukebox song components are supported (1.21+).
    pub fn supports_jukebox_songs(&self) -> bool {
        self.supports_jukebox_songs
    }

    /// Returns `true` if damage type registries are supported (1.19.4+).
    pub fn supports_damage_types(&self) -> bool {
        self.supports_damage_types
    }

    /// Resolve a [`MinecraftVersion`] into a [`VersionProfile`], returning an error
    /// if the version is not in the known table (i.e. `is_fallback` would be `true`).
    ///
    /// Use this in CI/release builds to prevent silently emitting packs for
    /// unverified Minecraft versions. For local experimentation, use
    /// [`resolve`](Self::resolve) which returns a conservative fallback instead.
    ///
    /// # Errors
    /// Returns [`VersionError::UnknownVersion`] for any version that is not
    /// explicitly listed in the known-version table, including future `26.x` series
    /// versions and future `1.x` minor versions not yet verified by Sand.
    ///
    /// # Examples
    /// ```
    /// use sand_core::version::{MinecraftVersion, VersionProfile};
    ///
    /// // Known version → OK
    /// let v = MinecraftVersion::parse("1.21.4").unwrap();
    /// assert!(VersionProfile::resolve_strict(&v).is_ok());
    ///
    /// // Unknown version → Err
    /// let v = MinecraftVersion::parse("26.99").unwrap();
    /// assert!(VersionProfile::resolve_strict(&v).is_err());
    /// ```
    pub fn resolve_strict(version: &MinecraftVersion) -> Result<Self, VersionError> {
        let profile = Self::resolve(version)?;
        if profile.is_fallback {
            return Err(VersionError::UnknownVersion {
                requested: version.to_string(),
                hint: "Add an explicit `pack_format` override in sand.toml, \
                       or use `VersionProfile::resolve` to accept a conservative \
                       fallback for local experimentation."
                    .to_string(),
            });
        }
        Ok(profile)
    }

    /// Return pack metadata for a datapack using this version profile.
    ///
    /// The returned value contains the exact `pack_format` to write to `pack.mcmeta`.
    /// When `is_fallback` is `true`, both formats are derived from the latest known
    /// version and the caller should warn that the output may not be validated.
    pub fn datapack_metadata(&self) -> PackMetadata {
        PackMetadata {
            pack_format: self.data_pack_format,
            is_fallback: self.is_fallback,
        }
    }

    /// Return pack metadata for a resource pack using this version profile.
    pub fn resourcepack_metadata(&self) -> PackMetadata {
        PackMetadata {
            pack_format: self.resource_pack_format,
            is_fallback: self.is_fallback,
        }
    }

    /// Query a named capability by string key.
    ///
    /// Useful for version-gating features without importing each flag name:
    /// ```
    /// use sand_core::version::{MinecraftVersion, VersionProfile};
    ///
    /// let v = MinecraftVersion::parse("1.21.4").unwrap();
    /// let p = VersionProfile::resolve(&v).unwrap();
    /// assert!(p.supports_feature("item_components"));
    /// assert!(!p.supports_feature("dialogs"));
    /// ```
    pub fn supports_feature(&self, feature: &str) -> bool {
        match feature {
            "dialogs" => self.supports_dialogs,
            "function_macros" => self.supports_function_macros,
            "predicates" => self.supports_predicates,
            "resource_pack_overlays" => self.supports_resource_pack_overlays,
            "trim_assets" => self.supports_trim_assets,
            "jukebox_songs" => self.supports_jukebox_songs,
            "damage_types" => self.supports_damage_types,
            "chat_types" => self.supports_chat_types,
            "enchantments" => self.supports_enchantments,
            "item_components" => self.supports_item_components,
            "data_components" => self.supports_data_components,
            "26_series" => self.supports_26_series,
            _ => false,
        }
    }
}

struct VersionCaps {
    data_fmt: u32,
    res_fmt: u32,
    item_components: bool,
    data_components: bool,
    dialogs: bool,
    function_macros: bool,
    predicates: bool,
    resource_pack_overlays: bool,
    trim_assets: bool,
    jukebox_songs: bool,
    damage_types: bool,
    chat_types: bool,
    enchantments: bool,
    is_fallback: bool,
}

impl Default for VersionCaps {
    /// All-features-enabled baseline used as a spread target by known-version arms.
    ///
    /// Do NOT use this as the fallback for unknown versions — use
    /// [`VersionCaps::conservative`] instead.
    fn default() -> Self {
        Self {
            data_fmt: 61,
            res_fmt: 46,
            item_components: true,
            data_components: true,
            dialogs: true,
            function_macros: true,
            predicates: true,
            resource_pack_overlays: true,
            trim_assets: true,
            jukebox_songs: true,
            damage_types: true,
            chat_types: true,
            enchantments: true,
            is_fallback: false,
        }
    }
}

impl VersionCaps {
    /// Conservative profile for any version not explicitly listed in the known table.
    ///
    /// All feature flags are `false`; pack formats default to the latest known
    /// values so that `pack.mcmeta` is at least structurally valid.  The caller
    /// must warn the user that output for this version is unverified.
    fn conservative() -> Self {
        Self {
            data_fmt: 61,
            res_fmt: 46,
            item_components: false,
            data_components: false,
            dialogs: false,
            function_macros: false,
            predicates: false,
            resource_pack_overlays: false,
            trim_assets: false,
            jukebox_songs: false,
            damage_types: false,
            chat_types: false,
            enchantments: false,
            is_fallback: true,
        }
    }
}

/// Look up version capabilities from (major, minor, patch).
fn lookup(major: u32, minor: u32, patch: u32) -> VersionCaps {
    match (major, minor, patch) {
        // ── 1.21.6+ — dialogs introduced ─────────────────────────────────────
        (1, 21, p) if p >= 6 => VersionCaps {
            data_fmt: 61,
            res_fmt: 46,
            dialogs: true,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.21.4-5 ─────────────────────────────────────────────────────────
        (1, 21, 4..=5) => VersionCaps {
            data_fmt: 61,
            res_fmt: 46,
            dialogs: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.21.2-3 ─────────────────────────────────────────────────────────
        (1, 21, 2..=3) => VersionCaps {
            data_fmt: 57,
            res_fmt: 42,
            dialogs: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.21.0-1 ─────────────────────────────────────────────────────────
        (1, 21, 0..=1) => VersionCaps {
            data_fmt: 48,
            res_fmt: 34,
            dialogs: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.20.5-6 ─────────────────────────────────────────────────────────
        (1, 20, 5..=6) => VersionCaps {
            data_fmt: 41,
            res_fmt: 32,
            dialogs: false,
            jukebox_songs: false,
            enchantments: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.20.3-4 ─────────────────────────────────────────────────────────
        (1, 20, 3..=4) => VersionCaps {
            data_fmt: 26,
            res_fmt: 22,
            item_components: false,
            dialogs: false,
            jukebox_songs: false,
            enchantments: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.20.2 ───────────────────────────────────────────────────────────
        (1, 20, 2) => VersionCaps {
            data_fmt: 18,
            res_fmt: 18,
            item_components: false,
            dialogs: false,
            jukebox_songs: false,
            enchantments: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.20.0-1 ─────────────────────────────────────────────────────────
        (1, 20, 0..=1) => VersionCaps {
            data_fmt: 15,
            res_fmt: 15,
            item_components: false,
            data_components: false,
            dialogs: false,
            function_macros: false,
            resource_pack_overlays: false,
            jukebox_songs: false,
            enchantments: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.19.4 ───────────────────────────────────────────────────────────
        (1, 19, 4) => VersionCaps {
            data_fmt: 12,
            res_fmt: 13,
            item_components: false,
            data_components: false,
            dialogs: false,
            function_macros: false,
            resource_pack_overlays: false,
            jukebox_songs: false,
            enchantments: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.19.0-3 ─────────────────────────────────────────────────────────
        (1, 19, 0..=3) => VersionCaps {
            data_fmt: 10,
            res_fmt: 12,
            item_components: false,
            data_components: false,
            dialogs: false,
            function_macros: false,
            resource_pack_overlays: false,
            trim_assets: false,
            jukebox_songs: false,
            enchantments: false,
            damage_types: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 26.x series — pack formats not verified; use conservative caps ────
        //    Until specific 26.x versions are mapped, treat all as unknown.
        //    Use VersionProfile::resolve_strict() to reject these outright.
        (26, _, _) => VersionCaps::conservative(),
        // ── future 1.x > 1.21 — conservative fallback ────────────────────────
        (1, minor, _) if minor > 21 => VersionCaps::conservative(),
        // ── anything else — conservative fallback ─────────────────────────────
        _ => VersionCaps::conservative(),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse ─────────────────────────────────────────────────────────────────

    #[test]
    fn parse_three_part_legacy() {
        let v = MinecraftVersion::parse("1.21.4").unwrap();
        assert_eq!(v.components(), Some((1, 21, 4)));
        assert!(v.is_121_series());
    }

    #[test]
    fn parse_two_part_legacy() {
        let v = MinecraftVersion::parse("1.21").unwrap();
        assert_eq!(v.components(), Some((1, 21, 0)));
    }

    #[test]
    fn parse_long_minor_legacy() {
        let v = MinecraftVersion::parse("1.21.11").unwrap();
        assert_eq!(v.components(), Some((1, 21, 11)));
    }

    #[test]
    fn parse_single_part_26() {
        let v = MinecraftVersion::parse("26").unwrap();
        assert_eq!(v.components(), Some((26, 0, 0)));
        assert!(v.is_26_series());
    }

    #[test]
    fn parse_two_part_26() {
        let v = MinecraftVersion::parse("26.1").unwrap();
        assert_eq!(v.components(), Some((26, 1, 0)));
        assert!(v.is_26_series());
    }

    #[test]
    fn parse_three_part_26() {
        let v = MinecraftVersion::parse("26.1.2").unwrap();
        assert_eq!(v.components(), Some((26, 1, 2)));
        assert!(v.is_26_series());
    }

    #[test]
    fn parse_latest() {
        let v = MinecraftVersion::parse("latest").unwrap();
        assert!(v.is_latest());
        assert!(!v.is_26_series());
        assert!(!v.is_121_series());
    }

    #[test]
    fn parse_invalid_alpha() {
        assert_eq!(
            MinecraftVersion::parse("abc"),
            Err(VersionError::ParseError("abc".to_string()))
        );
    }

    #[test]
    fn parse_invalid_1_foo() {
        assert_eq!(
            MinecraftVersion::parse("1.foo"),
            Err(VersionError::ParseError("1.foo".to_string()))
        );
    }

    #[test]
    fn parse_invalid_26_x() {
        assert_eq!(
            MinecraftVersion::parse("26.x"),
            Err(VersionError::ParseError("26.x".to_string()))
        );
    }

    #[test]
    fn parse_invalid_too_many_parts() {
        assert!(MinecraftVersion::parse("1.21.4.5").is_err());
    }

    // ── resolve ───────────────────────────────────────────────────────────────

    #[test]
    fn resolve_121_4() {
        let v = MinecraftVersion::parse("1.21.4").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert_eq!(p.data_pack_format, 61);
        assert_eq!(p.resource_pack_format, 46);
        assert!(p.supports_item_components);
        assert!(p.supports_data_components);
        assert!(!p.supports_26_series);
        assert!(!p.is_fallback);
    }

    #[test]
    fn resolve_121_11_future_fallback() {
        let v = MinecraftVersion::parse("1.21.11").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        // 1.21.11 is beyond the table; still resolves with fallback
        assert_eq!(p.data_pack_format, 61);
        assert_eq!(p.resource_pack_format, 46);
        assert!(!p.is_fallback);
    }

    #[test]
    fn resolve_26_series() {
        let v = MinecraftVersion::parse("26.1").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(p.supports_26_series);
        assert!(
            p.is_fallback,
            "26.x is conservative since no version is mapped yet"
        );
        assert!(
            !p.supports_item_components,
            "conservative profile has all features false"
        );
    }

    #[test]
    fn resolve_26_unknown_future() {
        let v = MinecraftVersion::parse("26.99").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(p.supports_26_series);
        assert!(p.is_fallback);
        assert!(
            !p.supports_dialogs,
            "unverified version must not claim dialog support"
        );
    }

    #[test]
    fn resolve_latest() {
        let v = MinecraftVersion::parse("latest").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(p.resolved_name.contains("latest"));
        assert_eq!(p.data_pack_format, 61);
    }

    #[test]
    fn display() {
        assert_eq!(
            MinecraftVersion::parse("1.21.4").unwrap().to_string(),
            "1.21.4"
        );
        assert_eq!(
            MinecraftVersion::parse("26.1").unwrap().to_string(),
            "26.1.0"
        );
        assert_eq!(
            MinecraftVersion::parse("latest").unwrap().to_string(),
            "latest"
        );
    }

    // ── Capability tests ──────────────────────────────────────────────────────

    #[test]
    fn dialogs_not_in_1_21_4() {
        let v = MinecraftVersion::parse("1.21.4").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(!p.supports_dialogs(), "1.21.4 predates dialogs (1.21.6)");
        assert!(!p.supports_feature("dialogs"));
    }

    #[test]
    fn dialogs_not_in_1_21_5() {
        let v = MinecraftVersion::parse("1.21.5").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(!p.supports_dialogs(), "1.21.5 predates dialogs");
    }

    #[test]
    fn dialogs_in_1_21_6() {
        let v = MinecraftVersion::parse("1.21.6").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(p.supports_dialogs(), "1.21.6 introduced dialogs");
    }

    #[test]
    fn dialogs_not_in_26x_unverified() {
        // 26.x is conservative until specific versions are mapped to exact formats.
        let v = MinecraftVersion::parse("26.1").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(
            !p.supports_dialogs(),
            "26.x is unverified — must not claim dialog support"
        );
    }

    #[test]
    fn function_macros_gated() {
        let old = MinecraftVersion::parse("1.20.1").unwrap();
        let p = VersionProfile::resolve(&old).unwrap();
        assert!(!p.supports_function_macros(), "1.20.1 has no macros");

        let new = MinecraftVersion::parse("1.20.2").unwrap();
        let p2 = VersionProfile::resolve(&new).unwrap();
        assert!(p2.supports_function_macros(), "1.20.2 added macros");
    }

    #[test]
    fn jukebox_songs_gated() {
        let old = MinecraftVersion::parse("1.20.6").unwrap();
        let p = VersionProfile::resolve(&old).unwrap();
        assert!(!p.supports_jukebox_songs(), "1.20.x has no jukebox songs");

        let new = MinecraftVersion::parse("1.21.0").unwrap();
        let p2 = VersionProfile::resolve(&new).unwrap();
        assert!(p2.supports_jukebox_songs(), "1.21+ has jukebox songs");
    }

    #[test]
    fn supports_feature_generic() {
        let v = MinecraftVersion::parse("1.21.4").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(p.supports_feature("item_components"));
        assert!(p.supports_feature("function_macros"));
        assert!(!p.supports_feature("dialogs"));
        assert!(!p.supports_feature("nonexistent_feature"));
    }

    #[test]
    fn capabilities_1_21_x() {
        let v = MinecraftVersion::parse("1.21.4").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(p.supports_item_components);
        assert!(p.supports_data_components);
        assert!(p.supports_function_macros);
        assert!(p.supports_predicates);
        assert!(p.supports_trim_assets);
        assert!(p.supports_jukebox_songs);
        assert!(p.supports_damage_types);
        assert!(p.supports_chat_types);
        assert!(p.supports_enchantments);
    }

    #[test]
    fn capabilities_26x_fallback() {
        let v = MinecraftVersion::parse("26.99").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(p.is_fallback);
        assert!(!p.supports_dialogs, "conservative profile: dialogs=false");
        assert!(p.supports_26_series);
    }

    // ── resolve_strict ────────────────────────────────────────────────────────

    #[test]
    fn strict_known_version_ok() {
        let v = MinecraftVersion::parse("1.21.4").unwrap();
        assert!(VersionProfile::resolve_strict(&v).is_ok());
    }

    #[test]
    fn strict_unknown_26x_fails() {
        let v = MinecraftVersion::parse("26.1").unwrap();
        let err = VersionProfile::resolve_strict(&v).unwrap_err();
        assert!(
            matches!(err, VersionError::UnknownVersion { .. }),
            "expected UnknownVersion, got {err:?}"
        );
    }

    #[test]
    fn strict_future_1x_fails() {
        let v = MinecraftVersion::parse("1.22.0").unwrap();
        let err = VersionProfile::resolve_strict(&v).unwrap_err();
        assert!(matches!(err, VersionError::UnknownVersion { .. }));
    }

    #[test]
    fn strict_latest_known_boundary_ok() {
        // 1.21.6+ is in the known table, so strict resolution should succeed.
        let v = MinecraftVersion::parse("1.21.6").unwrap();
        assert!(VersionProfile::resolve_strict(&v).is_ok());
    }

    // ── PackMetadata ──────────────────────────────────────────────────────────

    #[test]
    fn pack_metadata_known_datapack() {
        let v = MinecraftVersion::parse("1.21.4").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        let m = p.datapack_metadata();
        assert_eq!(m.pack_format, 61);
        assert!(!m.is_fallback);
    }

    #[test]
    fn pack_metadata_known_resourcepack() {
        let v = MinecraftVersion::parse("1.21.4").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        let m = p.resourcepack_metadata();
        assert_eq!(m.pack_format, 46);
        assert!(!m.is_fallback);
    }

    #[test]
    fn pack_metadata_oldest_profile_datapack() {
        let v = MinecraftVersion::parse("1.19.0").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        let m = p.datapack_metadata();
        assert_eq!(m.pack_format, 10);
        assert!(!m.is_fallback);
    }

    #[test]
    fn pack_metadata_fallback_is_flagged() {
        let v = MinecraftVersion::parse("26.99").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        let m = p.datapack_metadata();
        assert!(m.is_fallback);
    }
}
