#![forbid(unsafe_code)]

//! Shared Minecraft version anchors and capability types used by Sand crates
//! that cannot depend on `sand-core` without creating build-time dependency
//! cycles.
//!
//! [`ComponentFeature`] and [`VersionCaps`] live here so that `sand-components`
//! can declare and query version requirements without importing
//! `sand_core::version::VersionProfile` (which would create a cycle).

/// The latest Minecraft version Sand's bundled version table was verified against.
///
/// This is the **export/profile anchor**: it is the version
/// `VersionProfile::resolve("latest")` resolves to, and it drives pack
/// metadata (`pack.mcmeta`) and version-sensitive feature flags. It is *not*
/// necessarily the same version used to run `sand-build` codegen for local
/// `sand-core` builds/tests — see [`DEFAULT_CODEGEN_VERSION`].
pub const LATEST_KNOWN: &str = "26.2";

/// The established generated-API CI baseline.
///
/// CI intentionally exercises this older, known-good codegen target as well
/// as [`LATEST_KNOWN`] so support for the stable baseline cannot regress while
/// the bundled version table advances.
pub const CI_STABLE_CODEGEN_VERSION: &str = "1.21.4";

/// The default Minecraft version `sand-core/build.rs` uses to run `sand-build`
/// codegen when `SAND_MC_VERSION` is unset.
///
/// This is the **codegen anchor**, kept deliberately separate from
/// [`LATEST_KNOWN`] so the two concerns do not get conflated:
///
/// - [`LATEST_KNOWN`] answers "which version profile do exported packs and
///   feature flags target by default?"
/// - `DEFAULT_CODEGEN_VERSION` answers "which verified, codegen-available
///   Minecraft server jar should local `cargo test -p sand-core --lib` use to
///   generate command/registry/block-state Rust APIs?"
///
/// The value MUST be a verified, codegen-available version: `sand-build` must
/// be able to download/cache its server jar and run the Minecraft data
/// generator to produce non-placeholder `commands.rs`, `registries.rs`, and
/// `block_states.rs`. It need not equal [`LATEST_KNOWN`]; when they differ,
/// [`LATEST_KNOWN`] is the export/profile target and `DEFAULT_CODEGEN_VERSION`
/// is the build-time codegen target.
///
/// If codegen fails, `sand-core/build.rs` fails immediately with an actionable
/// message (no silent placeholders). Set `SAND_ALLOW_PLACEHOLDER_CODEGEN=1` to
/// explicitly opt into placeholder files that compile but fail
/// `generated_api_health`. Changing this value requires confirming the new
/// target is codegen-available in the default local and CI environments.
pub const DEFAULT_CODEGEN_VERSION: &str = "1.21.11";

// ── Component capability identifiers ───────────────────────────────────────────

/// A Minecraft datapack component feature that may be gated by version.
///
/// Components declare their requirements via
/// [`DatapackComponent::required_features`](sand_components::component::DatapackComponent::required_features),
/// and the export layer checks them against [`VersionCaps`] resolved from the
/// target `VersionProfile`.
///
/// The variants mirror the `supports_*` fields of `sand_core::version::VersionProfile`.
/// Keeping them in `sand-version` avoids a dependency cycle between
/// `sand-components` and `sand-core`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentFeature {
    /// Data-driven dialogs (1.21.6+ / 26.x).
    Dialogs,
    /// Jukebox song components (1.21+).
    JukeboxSongs,
    /// Damage type registries (1.19.4+).
    DamageTypes,
    /// Chat type registries (1.19+).
    ChatTypes,
    /// Enchantment data components (1.21+).
    Enchantments,
    /// Armor trim assets — trim material and trim pattern components (1.19.4+).
    TrimAssets,
}

impl ComponentFeature {
    /// Human-readable feature name used in diagnostics.
    pub fn name(self) -> &'static str {
        match self {
            Self::Dialogs => "dialogs",
            Self::JukeboxSongs => "jukebox_songs",
            Self::DamageTypes => "damage_types",
            Self::ChatTypes => "chat_types",
            Self::Enchantments => "enchantments",
            Self::TrimAssets => "trim_assets",
        }
    }

    /// All feature variants, in a stable order.
    pub const ALL: &'static [ComponentFeature] = &[
        Self::Dialogs,
        Self::JukeboxSongs,
        Self::DamageTypes,
        Self::ChatTypes,
        Self::Enchantments,
        Self::TrimAssets,
    ];
}

/// Resolved version capability set used to gate component features.
///
/// This is a slimmed-down, cycle-safe mirror of
/// `sand_core::version::VersionProfile`'s `supports_*` fields. `sand-core`
/// produces it via `VersionProfile::caps()`; `sand-components` and the export
/// layer consume it without depending on `sand-core`.
///
/// For fallback/unknown profiles, all feature flags are `false`, matching the
/// conservative policy: reject version-gated components unless the user
/// explicitly targets a known exact profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionCaps {
    supports_dialogs: bool,
    supports_jukebox_songs: bool,
    supports_damage_types: bool,
    supports_chat_types: bool,
    supports_enchantments: bool,
    supports_trim_assets: bool,
}

impl VersionCaps {
    /// Create a `VersionCaps` where all features are enabled.
    ///
    /// Used by the compatibility (unprofiled) export path so existing
    /// callers retain their prior behavior.
    pub fn all_enabled() -> Self {
        Self {
            supports_dialogs: true,
            supports_jukebox_songs: true,
            supports_damage_types: true,
            supports_chat_types: true,
            supports_enchantments: true,
            supports_trim_assets: true,
        }
    }

    /// Create a `VersionCaps` where all features are disabled (fallback policy).
    pub fn all_disabled() -> Self {
        Self {
            supports_dialogs: false,
            supports_jukebox_songs: false,
            supports_damage_types: false,
            supports_chat_types: false,
            supports_enchantments: false,
            supports_trim_assets: false,
        }
    }

    /// Check whether a specific feature is supported by this capability set.
    pub fn supports(&self, feature: ComponentFeature) -> bool {
        match feature {
            ComponentFeature::Dialogs => self.supports_dialogs,
            ComponentFeature::JukeboxSongs => self.supports_jukebox_songs,
            ComponentFeature::DamageTypes => self.supports_damage_types,
            ComponentFeature::ChatTypes => self.supports_chat_types,
            ComponentFeature::Enchantments => self.supports_enchantments,
            ComponentFeature::TrimAssets => self.supports_trim_assets,
        }
    }

    /// Create a `VersionCaps` from individual feature flags.
    ///
    /// Used by `sand-core::VersionProfile::caps()`.
    pub fn from_flags(
        supports_dialogs: bool,
        supports_jukebox_songs: bool,
        supports_damage_types: bool,
        supports_chat_types: bool,
        supports_enchantments: bool,
        supports_trim_assets: bool,
    ) -> Self {
        Self {
            supports_dialogs,
            supports_jukebox_songs,
            supports_damage_types,
            supports_chat_types,
            supports_enchantments,
            supports_trim_assets,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_enabled_supports_everything() {
        let caps = VersionCaps::all_enabled();
        for feature in ComponentFeature::ALL {
            assert!(caps.supports(*feature), "{feature:?} should be enabled");
        }
    }

    #[test]
    fn all_disabled_supports_nothing() {
        let caps = VersionCaps::all_disabled();
        for feature in ComponentFeature::ALL {
            assert!(!caps.supports(*feature), "{feature:?} should be disabled");
        }
    }

    #[test]
    fn feature_name_is_stable() {
        assert_eq!(ComponentFeature::Dialogs.name(), "dialogs");
        assert_eq!(ComponentFeature::JukeboxSongs.name(), "jukebox_songs");
        assert_eq!(ComponentFeature::DamageTypes.name(), "damage_types");
        assert_eq!(ComponentFeature::ChatTypes.name(), "chat_types");
        assert_eq!(ComponentFeature::Enchantments.name(), "enchantments");
        assert_eq!(ComponentFeature::TrimAssets.name(), "trim_assets");
    }

    #[test]
    fn from_flags_respects_individual_values() {
        let caps = VersionCaps::from_flags(true, false, true, false, true, false);
        assert!(caps.supports(ComponentFeature::Dialogs));
        assert!(!caps.supports(ComponentFeature::JukeboxSongs));
        assert!(caps.supports(ComponentFeature::DamageTypes));
        assert!(!caps.supports(ComponentFeature::ChatTypes));
        assert!(caps.supports(ComponentFeature::Enchantments));
        assert!(!caps.supports(ComponentFeature::TrimAssets));
    }

    #[test]
    fn codegen_ci_targets_are_explicit_verified_versions() {
        assert_eq!(CI_STABLE_CODEGEN_VERSION, "1.21.4");
        assert!(!LATEST_KNOWN.is_empty());
        assert_ne!(CI_STABLE_CODEGEN_VERSION, "latest");
        assert_ne!(LATEST_KNOWN, "latest");
    }

    #[test]
    fn rust_workflow_resolves_codegen_targets_from_this_crate() {
        let workflow = include_str!("../../.github/workflows/rust.yml");
        assert!(workflow.contains("codegen-ci-version -- stable"));
        assert!(workflow.contains("codegen-ci-version -- latest"));
        assert!(workflow.contains("SAND_STRICT_CODEGEN: \"1\""));
        assert!(workflow.contains("Generated API health (stable"));
        assert!(workflow.contains("Generated API health (latest verified"));
        assert!(workflow.contains("Set up Java 21 for stable codegen"));
        assert!(workflow.contains("Set up Java 25 for latest verified codegen"));
    }
}
