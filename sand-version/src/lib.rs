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

/// The oldest Minecraft version Sand intentionally promises compatibility
/// with, and the CI job that verifies that promise still codegens.
///
/// This is an explicit compatibility/profile-boundary target, not the
/// canonical one: `1.21.4` fixtures cover a known rendering-branch boundary
/// (see `sand-core/tests/advancement_version_export.rs` and the
/// profile/fallback tests in `version.rs`). Canonical fixtures, examples,
/// and the default local codegen target are [`LATEST_KNOWN`]/
/// [`DEFAULT_CODEGEN_VERSION`] (Minecraft 26.2) — 1.21.4 is retained here
/// only so that boundary cannot silently regress, not because it is the
/// implicit default format.
pub const CI_STABLE_CODEGEN_VERSION: &str = "1.21.4";

/// Java runtimes required by the verified vanilla-server validation matrix.
pub const CI_STABLE_JAVA_VERSION: &str = "21";
pub const CI_LATEST_JAVA_VERSION: &str = "25";

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
pub const DEFAULT_CODEGEN_VERSION: &str = LATEST_KNOWN;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::CommandProfile",
    aliases = ["sand::cmd::CommandProfile", "sand::prelude::cmd::CommandProfile"],
    module = "sand::command",
    summary = "Cycle-safe command rendering context shared by `sand-commands` and `sand-core`.",
    context = "Cycle-safe command rendering context shared by `sand-commands` and `sand-core`. This deliberately carries only version identity today. Command families can add narrowly-scoped capability flags as their vanilla syntax diverges; callers should not infer support merely from the version string.",
    minecraft = "This deliberately carries only version identity today. Command families can add narrowly-scoped capability flags as their vanilla syntax diverges; callers should not infer support merely from the version string.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::CommandProfile;",
)]
/// Cycle-safe command rendering context shared by `sand-commands` and
/// `sand-core`.
///
/// This deliberately carries only version identity today. Command families
/// can add narrowly-scoped capability flags as their vanilla syntax diverges;
/// callers should not infer support merely from the version string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandProfile {
    requested_version: String,
    is_fallback: bool,
}

impl CommandProfile {
    /// Construct a command profile for a resolved Minecraft target.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::CommandProfile::new",
        aliases = ["sand::cmd::CommandProfile::new", "sand::prelude::cmd::CommandProfile::new"],
        module = "sand::command",
        kind = "method",
        summary = "Construct a command profile for a resolved Minecraft target.",
        context = "Construct a command profile for a resolved Minecraft target. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(requested_version = "`requested_version` is used when constructing a command profile for a resolved Minecraft target.", is_fallback = "`is_fallback` provides the switch that enables or disables the behavior used to construct a command profile for a resolved Minecraft target."),
        returns = "A `CommandProfile` representing a command profile for a resolved Minecraft target.",
        example = "use sand::prelude::*;\n\nfn demonstrate(requested_version: impl Into < String >, is_fallback: bool)  {\n    let command_profile = sand::command::CommandProfile::new(requested_version, is_fallback);\n}",
    )]
    pub fn new(requested_version: impl Into<String>, is_fallback: bool) -> Self {
        Self {
            requested_version: requested_version.into(),
            is_fallback,
        }
    }

    /// Compatibility profile used by direct command rendering without project
    /// configuration. Exporters should pass the project's resolved profile.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::CommandProfile::unprofiled",
        aliases = ["sand::cmd::CommandProfile::unprofiled", "sand::prelude::cmd::CommandProfile::unprofiled"],
        module = "sand::command",
        kind = "method",
        summary = "Compatibility profile used by direct command rendering without project configuration. Exporters should pass the project's resolved profile.",
        context = "Compatibility profile used by direct command rendering without project configuration. Exporters should pass the project's resolved profile. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `CommandProfile` configured for compatibility profile used by direct command rendering without project configuration. Exporters should pass the project's resolved profile.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let command_profile = sand::command::CommandProfile::unprofiled();\n}",
    )]
    pub fn unprofiled() -> Self {
        Self::new(LATEST_KNOWN, false)
    }

    /// Version requested by the project.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::CommandProfile::requested_version",
        aliases = ["sand::cmd::CommandProfile::requested_version", "sand::prelude::cmd::CommandProfile::requested_version"],
        module = "sand::command",
        kind = "method",
        summary = "Version requested by the project.",
        context = "Version requested by the project. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The string value produced to version requested by the project.",
        example = "use sand::prelude::*;\n\nfn demonstrate(command_profile_value: &sand::command::CommandProfile)  {\n    let requested_version = command_profile_value.requested_version();\n}",
    )]
    pub fn requested_version(&self) -> &str {
        &self.requested_version
    }

    /// Whether resolution used Sand's conservative fallback profile.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::CommandProfile::is_fallback",
        aliases = ["sand::cmd::CommandProfile::is_fallback", "sand::prelude::cmd::CommandProfile::is_fallback"],
        module = "sand::command",
        kind = "method",
        summary = "Whether resolution used Sand's conservative fallback profile.",
        context = "Whether resolution used Sand's conservative fallback profile. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "`true` when the documented condition holds to determine whether resolution used Sand's conservative fallback profile; otherwise `false`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(command_profile_value: &sand::command::CommandProfile)  {\n    let is_is_fallback = command_profile_value.is_fallback();\n}",
    )]
    pub fn is_fallback(&self) -> bool {
        self.is_fallback
    }

    /// Whether this resolved command target is at least the given Java release.
    ///
    /// Unknown/fallback profiles are conservative and never claim support.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::CommandProfile::is_at_least",
        aliases = ["sand::cmd::CommandProfile::is_at_least", "sand::prelude::cmd::CommandProfile::is_at_least"],
        module = "sand::command",
        kind = "method",
        summary = "Whether this resolved command target is at least the given Java release.",
        context = "Whether this resolved command target is at least the given Java release. Unknown/fallback profiles are conservative and never claim support.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(major = "`major` is the major considered when determining whether this resolved command target is at least the given Java release.", minor = "`minor` is the minor considered when determining whether this resolved command target is at least the given Java release.", patch = "`patch` is the patch considered when determining whether this resolved command target is at least the given Java release."),
        returns = "`true` when the documented condition holds to determine whether this resolved command target is at least the given Java release; otherwise `false`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(command_profile_value: &sand::command::CommandProfile, major: u32, minor: u32, patch: u32)  {\n    let is_is_at_least = command_profile_value.is_at_least(major, minor, patch);\n}",
    )]
    pub fn is_at_least(&self, major: u32, minor: u32, patch: u32) -> bool {
        if self.is_fallback {
            return false;
        }
        let value = if self.requested_version == "latest" {
            LATEST_KNOWN
        } else {
            &self.requested_version
        };
        let mut parts = value.split('.').map(|part| part.parse::<u32>());
        let Some(Ok(actual_major)) = parts.next() else {
            return false;
        };
        let actual_minor = parts.next().transpose().ok().flatten().unwrap_or(0);
        let actual_patch = parts.next().transpose().ok().flatten().unwrap_or(0);
        (actual_major, actual_minor, actual_patch) >= (major, minor, patch)
    }
}

// ── Component capability identifiers ───────────────────────────────────────────

/// A Minecraft datapack component feature that may be gated by version.
///
/// Components declare their requirements via
/// `DatapackComponent::required_features`,
/// and the export layer checks them against [`VersionCaps`] resolved from the
/// target `VersionProfile`.
///
/// The variants mirror the `supports_*` fields of `sand_core::version::VersionProfile`.
/// Keeping them in `sand-version` avoids a dependency cycle between
/// `sand-components` and `sand-core`.
///
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::version::ComponentFeature",
    module = "sand::version",
    summary = "A Minecraft datapack component feature that may be gated by version.",
    context = "A Minecraft datapack component feature that may be gated by version. Components declare their requirements via `DatapackComponent::required_features`, and the export layer checks them against [`VersionCaps`] resolved from the target `VersionProfile`. The variants mirror the `supports_*` fields of `sand::version::VersionProfile`. Keeping them in `sand-version` avoids a dependency cycle between `sand-components` and `sand-core`.",
    minecraft = "Components declare their requirements via `DatapackComponent::required_features`, and the export layer checks them against [`VersionCaps`] resolved from the target `VersionProfile`.",
    use_when = ["Adapting authored resources or integrations to an explicitly selected Minecraft target"],
    avoid_when = ["Ordinary datapack code can rely on the target selected in sand.toml"],
    example = "use sand::version::ComponentFeature;",
    variants(AnimalVariants = "Biome-scoped animal variant registries — `chicken_variant`, `cow_variant`, `pig_variant` (1.21.5+).", ChatTypes = "Chat type registries (1.19+).", DamageTypes = "Damage type registries (1.19.4+).", Dialogs = "Data-driven dialogs (1.21.6+ / 26.x).", Enchantments = "Enchantment data components (1.21+).", ItemComponents = "Item data components — the 1.20.5+ component system (`minecraft:custom_data`, `minecraft:item_name`, etc.). Gates component-bearing recipe results and other JSON payloads that embed structured item components.", JukeboxSongs = "Jukebox song components (1.21+).", TrimAssets = "Armor trim assets — trim material and trim pattern components (1.19.4+).", VillagerTrades = "Data-driven Villager/Wandering Trader trade registries — `villager_trade` and `trade_set` (26.1+)."),
)]
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
    /// Item data components — the 1.20.5+ component system (`minecraft:custom_data`,
    /// `minecraft:item_name`, etc.). Gates component-bearing recipe results and
    /// other JSON payloads that embed structured item components.
    ItemComponents,
    /// Biome-scoped animal variant registries — `chicken_variant`,
    /// `cow_variant`, `pig_variant` (1.21.5+).
    AnimalVariants,
    /// Data-driven Villager/Wandering Trader trade registries —
    /// `villager_trade` and `trade_set` (26.1+).
    VillagerTrades,
}

impl ComponentFeature {
    /// Human-readable feature name used in diagnostics.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::version::ComponentFeature::name",
        module = "sand::version",
        kind = "method",
        summary = "Human-readable feature name used in diagnostics.",
        context = "Human-readable feature name used in diagnostics. This version API lets reusable authoring and tooling make the same capability decisions as Sand's profile-aware component exporter.",
        minecraft = "Capability checks describe the data-driven features accepted by the selected Minecraft Java Edition target before pack output is written.",
        use_when = ["Adapting authored resources or integrations to an explicitly selected Minecraft target"],
        avoid_when = ["Ordinary datapack code can rely on the target selected in sand.toml"],
        returns = "The string value produced to human-readable feature name used in diagnostics.",
        example = "use sand::prelude::*;\n\nfn demonstrate(component_feature_value: sand::version::ComponentFeature)  {\n    let name = component_feature_value.name();\n}",
    )]
    pub fn name(self) -> &'static str {
        match self {
            Self::Dialogs => "dialogs",
            Self::JukeboxSongs => "jukebox_songs",
            Self::DamageTypes => "damage_types",
            Self::ChatTypes => "chat_types",
            Self::Enchantments => "enchantments",
            Self::TrimAssets => "trim_assets",
            Self::ItemComponents => "item_components",
            Self::AnimalVariants => "animal_variants",
            Self::VillagerTrades => "villager_trades",
        }
    }

    /// All feature variants, in a stable order.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::version::ComponentFeature::ALL",
        module = "sand::version",
        kind = "associated_const",
        summary = "All feature variants, in a stable order.",
        context = "All feature variants, in a stable order. This version API lets reusable authoring and tooling make the same capability decisions as Sand's profile-aware component exporter.",
        minecraft = "Capability checks describe the data-driven features accepted by the selected Minecraft Java Edition target before pack output is written.",
        use_when = ["Adapting authored resources or integrations to an explicitly selected Minecraft target"],
        avoid_when = ["Ordinary datapack code can rely on the target selected in sand.toml"],
        example = "use sand::version::ComponentFeature;",
    )]
    pub const ALL: &'static [ComponentFeature] = &[
        Self::Dialogs,
        Self::JukeboxSongs,
        Self::DamageTypes,
        Self::ChatTypes,
        Self::Enchantments,
        Self::TrimAssets,
        Self::ItemComponents,
        Self::AnimalVariants,
        Self::VillagerTrades,
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
///
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::version::VersionCaps",
    module = "sand::version",
    summary = "Resolved version capability set used to gate component features.",
    context = "Resolved version capability set used to gate component features. This is a slimmed-down, cycle-safe mirror of `sand::version::VersionProfile`'s `supports_*` fields. `sand-core` produces it via `VersionProfile::caps()`; `sand-components` and the export layer consume it without depending on `sand-core`. For fallback/unknown profiles, all feature flags are `false`, matching the conservative policy: reject version-gated components unless the user explicitly targets a known exact profile.",
    minecraft = "This is a slimmed-down, cycle-safe mirror of `sand::version::VersionProfile`'s `supports_*` fields. `sand-core` produces it via `VersionProfile::caps()`; `sand-components` and the export layer consume it without depending on `sand-core`.",
    use_when = ["Adapting authored resources or integrations to an explicitly selected Minecraft target"],
    avoid_when = ["Ordinary datapack code can rely on the target selected in sand.toml"],
    example = "use sand::version::VersionCaps;",
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionCaps {
    requested_version: String,
    is_fallback: bool,
    supports_dialogs: bool,
    supports_jukebox_songs: bool,
    supports_damage_types: bool,
    supports_chat_types: bool,
    supports_enchantments: bool,
    supports_trim_assets: bool,
    supports_item_components: bool,
    supports_animal_variants: bool,
    supports_villager_trades: bool,
}

impl VersionCaps {
    /// Create a `VersionCaps` where all features are enabled.
    ///
    /// Used by the compatibility (unprofiled) export path so existing
    /// callers retain their prior behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::version::VersionCaps::all_enabled",
        module = "sand::version",
        kind = "method",
        summary = "Create a `VersionCaps` where all features are enabled.",
        context = "Create a `VersionCaps` where all features are enabled. Used by the compatibility (unprofiled) export path so existing callers retain their prior behavior.",
        minecraft = "Used by the compatibility (unprofiled) export path so existing callers retain their prior behavior.",
        use_when = ["Adapting authored resources or integrations to an explicitly selected Minecraft target"],
        avoid_when = ["Ordinary datapack code can rely on the target selected in sand.toml"],
        returns = "A `VersionCaps` with every modeled feature enabled.",
        example = "let caps = sand::version::VersionCaps::all_enabled();",
    )]
    pub fn all_enabled() -> Self {
        Self {
            requested_version: LATEST_KNOWN.to_string(),
            is_fallback: false,
            supports_dialogs: true,
            supports_jukebox_songs: true,
            supports_damage_types: true,
            supports_chat_types: true,
            supports_enchantments: true,
            supports_trim_assets: true,
            supports_item_components: true,
            supports_animal_variants: true,
            supports_villager_trades: true,
        }
    }

    /// Create a `VersionCaps` where all features are disabled (fallback policy).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::version::VersionCaps::all_disabled",
        module = "sand::version",
        kind = "method",
        summary = "Create a `VersionCaps` where all features are disabled (fallback policy).",
        context = "Create a `VersionCaps` where all features are disabled (fallback policy). This version API lets reusable authoring and tooling make the same capability decisions as Sand's profile-aware component exporter.",
        minecraft = "Capability checks describe the data-driven features accepted by the selected Minecraft Java Edition target before pack output is written.",
        use_when = ["Adapting authored resources or integrations to an explicitly selected Minecraft target"],
        avoid_when = ["Ordinary datapack code can rely on the target selected in sand.toml"],
        returns = "A `VersionCaps` with every modeled feature disabled for fallback behavior.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let version_caps = sand::version::VersionCaps::all_disabled();\n}",
    )]
    pub fn all_disabled() -> Self {
        Self {
            requested_version: "1.18".to_string(),
            is_fallback: true,
            supports_dialogs: false,
            supports_jukebox_songs: false,
            supports_damage_types: false,
            supports_chat_types: false,
            supports_enchantments: false,
            supports_trim_assets: false,
            supports_item_components: false,
            supports_animal_variants: false,
            supports_villager_trades: false,
        }
    }

    /// Set whether biome-scoped animal variant registries (`chicken_variant`,
    /// `cow_variant`, `pig_variant`; 1.21.5+) are supported.
    ///
    /// A separate builder method (rather than a constructor parameter) keeps
    /// [`VersionCaps::from_flags`]/[`VersionCaps::from_profile_flags`] call
    /// sites stable as new narrowly-scoped features are added.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::version::VersionCaps::with_animal_variants",
        module = "sand::version",
        kind = "method",
        summary = "Set whether biome-scoped animal variant registries (`chicken_variant`, `cow_variant`, `pig_variant`; 1.21.5+) are supported.",
        context = "Set whether biome-scoped animal variant registries (`chicken_variant`, `cow_variant`, `pig_variant`; 1.21.5+) are supported. A separate builder method (rather than a constructor parameter) keeps [`VersionCaps::from_flags`]/[`VersionCaps::from_profile_flags`] call sites stable as new narrowly-scoped features are added.",
        minecraft = "Capability checks describe the data-driven features accepted by the selected Minecraft Java Edition target before pack output is written.",
        use_when = ["Adapting authored resources or integrations to an explicitly selected Minecraft target"],
        avoid_when = ["Ordinary datapack code can rely on the target selected in sand.toml"],
        params(value = "`value` provides the value being applied or compared used to set whether biome-scoped animal variant registries (`chicken_variant`, `cow_variant`, `pig_variant`; 1.21.5+) are supported."),
        returns = "The `VersionCaps` value with the documented change applied to set whether biome-scoped animal variant registries (`chicken_variant`, `cow_variant`, `pig_variant`; 1.21.5+) are supported.",
        example = "use sand::prelude::*;\n\nfn demonstrate(version_caps_value: sand::version::VersionCaps, value: bool)  {\n    let updated_version_caps = version_caps_value.with_animal_variants(value);\n}",
    )]
    pub fn with_animal_variants(mut self, value: bool) -> Self {
        self.supports_animal_variants = value;
        self
    }

    /// Set whether the data-driven Villager/Wandering Trader trade
    /// registries (`villager_trade`, `trade_set`; 26.1+) are supported.
    ///
    /// Follows the same builder-method pattern as
    /// [`VersionCaps::with_animal_variants`] for the same reason: it keeps
    /// [`VersionCaps::from_flags`]/[`VersionCaps::from_profile_flags`] call
    /// sites stable.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::version::VersionCaps::with_villager_trades",
        module = "sand::version",
        kind = "method",
        summary = "Set whether the data-driven Villager/Wandering Trader trade registries (`villager_trade`, `trade_set`; 26.1+) are supported.",
        context = "Set whether the data-driven Villager/Wandering Trader trade registries (`villager_trade`, `trade_set`; 26.1+) are supported. Follows the same builder-method pattern as [`VersionCaps::with_animal_variants`] for the same reason: it keeps [`VersionCaps::from_flags`]/[`VersionCaps::from_profile_flags`] call sites stable.",
        minecraft = "Capability checks describe the data-driven features accepted by the selected Minecraft Java Edition target before pack output is written.",
        use_when = ["Adapting authored resources or integrations to an explicitly selected Minecraft target"],
        avoid_when = ["Ordinary datapack code can rely on the target selected in sand.toml"],
        params(value = "`value` provides the value being applied or compared used to set whether the data-driven Villager/Wandering Trader trade registries (`villager_trade`, `trade_set`; 26.1+) are supported."),
        returns = "The `VersionCaps` value with the documented change applied to set whether the data-driven Villager/Wandering Trader trade registries (`villager_trade`, `trade_set`; 26.1+) are supported.",
        example = "use sand::prelude::*;\n\nfn demonstrate(version_caps_value: sand::version::VersionCaps, value: bool)  {\n    let updated_version_caps = version_caps_value.with_villager_trades(value);\n}",
    )]
    pub fn with_villager_trades(mut self, value: bool) -> Self {
        self.supports_villager_trades = value;
        self
    }

    /// Check whether a specific feature is supported by this capability set.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::version::VersionCaps::supports",
        module = "sand::version",
        kind = "method",
        summary = "Check whether a specific feature is supported by this capability set.",
        context = "Check whether a specific feature is supported by this capability set. This version API lets reusable authoring and tooling make the same capability decisions as Sand's profile-aware component exporter.",
        minecraft = "Capability checks describe the data-driven features accepted by the selected Minecraft Java Edition target before pack output is written.",
        use_when = ["Adapting authored resources or integrations to an explicitly selected Minecraft target"],
        avoid_when = ["Ordinary datapack code can rely on the target selected in sand.toml"],
        params(feature = "`feature` is the feature checked to determine whether a specific feature is supported by this capability set."),
        returns = "`true` when the documented condition holds to check whether a specific feature is supported by this capability set; otherwise `false`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(version_caps_value: &sand::version::VersionCaps, feature: sand::version::ComponentFeature)  {\n    let is_supports = version_caps_value.supports(feature);\n}",
    )]
    pub fn supports(&self, feature: ComponentFeature) -> bool {
        match feature {
            ComponentFeature::Dialogs => self.supports_dialogs,
            ComponentFeature::JukeboxSongs => self.supports_jukebox_songs,
            ComponentFeature::DamageTypes => self.supports_damage_types,
            ComponentFeature::ChatTypes => self.supports_chat_types,
            ComponentFeature::Enchantments => self.supports_enchantments,
            ComponentFeature::TrimAssets => self.supports_trim_assets,
            ComponentFeature::ItemComponents => self.supports_item_components,
            ComponentFeature::AnimalVariants => self.supports_animal_variants,
            ComponentFeature::VillagerTrades => self.supports_villager_trades,
        }
    }

    /// Create an unprofiled `VersionCaps` from individual feature flags.
    ///
    /// This compatibility constructor retains the pre-profile API. Schema
    /// consumers treat it as the latest known target, matching unprofiled
    /// component export behavior.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::version::VersionCaps::from_flags",
        module = "sand::version",
        kind = "method",
        summary = "Create an unprofiled `VersionCaps` from individual feature flags.",
        context = "Create an unprofiled `VersionCaps` from individual feature flags. This compatibility constructor retains the pre-profile API. Schema consumers treat it as the latest known target, matching unprofiled component export behavior.",
        minecraft = "This compatibility constructor retains the pre-profile API. Schema consumers treat it as the latest known target, matching unprofiled component export behavior.",
        use_when = ["Adapting authored resources or integrations to an explicitly selected Minecraft target"],
        avoid_when = ["Ordinary datapack code can rely on the target selected in sand.toml"],
        params(supports_dialogs = "`supports_dialogs` provides the switch that enables or disables the behavior used to create an unprofiled `VersionCaps` from individual feature flags.", supports_jukebox_songs = "`supports_jukebox_songs` provides the switch that enables or disables the behavior used to create an unprofiled `VersionCaps` from individual feature flags.", supports_damage_types = "`supports_damage_types` provides the switch that enables or disables the behavior used to create an unprofiled `VersionCaps` from individual feature flags.", supports_chat_types = "`supports_chat_types` provides the switch that enables or disables the behavior used to create an unprofiled `VersionCaps` from individual feature flags.", supports_enchantments = "`supports_enchantments` provides the switch that enables or disables the behavior used to create an unprofiled `VersionCaps` from individual feature flags.", supports_trim_assets = "`supports_trim_assets` provides the switch that enables or disables the behavior used to create an unprofiled `VersionCaps` from individual feature flags.", supports_item_components = "`supports_item_components` provides the switch that enables or disables the behavior used to create an unprofiled `VersionCaps` from individual feature flags."),
        returns = "A `VersionCaps` representing an unprofiled `VersionCaps` from individual feature flags.",
        example = "use sand::prelude::*;\n\nfn demonstrate(supports_dialogs: bool, supports_jukebox_songs: bool, supports_damage_types: bool, supports_chat_types: bool, supports_enchantments: bool, supports_trim_assets: bool, supports_item_components: bool)  {\n    let version_caps = sand::version::VersionCaps::from_flags(supports_dialogs, supports_jukebox_songs, supports_damage_types, supports_chat_types, supports_enchantments, supports_trim_assets, supports_item_components);\n}",
    )]
    #[allow(clippy::too_many_arguments)]
    pub fn from_flags(
        supports_dialogs: bool,
        supports_jukebox_songs: bool,
        supports_damage_types: bool,
        supports_chat_types: bool,
        supports_enchantments: bool,
        supports_trim_assets: bool,
        supports_item_components: bool,
    ) -> Self {
        Self::from_profile_flags(
            LATEST_KNOWN,
            false,
            supports_dialogs,
            supports_jukebox_songs,
            supports_damage_types,
            supports_chat_types,
            supports_enchantments,
            supports_trim_assets,
            supports_item_components,
        )
    }

    /// Create a `VersionCaps` for a concrete resolved target profile.
    ///
    /// Used by `sand-core::VersionProfile::caps()` so schema consumers can
    /// distinguish targets that share the same feature flags.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::version::VersionCaps::from_profile_flags",
        module = "sand::version",
        kind = "method",
        summary = "Create a `VersionCaps` for a concrete resolved target profile.",
        context = "Create a `VersionCaps` for a concrete resolved target profile. Used by `sand-core::VersionProfile::caps()` so schema consumers can distinguish targets that share the same feature flags.",
        minecraft = "Capability checks describe the data-driven features accepted by the selected Minecraft Java Edition target before pack output is written.",
        use_when = ["Adapting authored resources or integrations to an explicitly selected Minecraft target"],
        avoid_when = ["Ordinary datapack code can rely on the target selected in sand.toml"],
        params(requested_version = "`requested_version` is used when creating a `VersionCaps` for a concrete resolved target profile.", is_fallback = "`is_fallback` provides the switch that enables or disables the behavior used to create a `VersionCaps` for a concrete resolved target profile.", supports_dialogs = "`supports_dialogs` provides the switch that enables or disables the behavior used to create a `VersionCaps` for a concrete resolved target profile.", supports_jukebox_songs = "`supports_jukebox_songs` provides the switch that enables or disables the behavior used to create a `VersionCaps` for a concrete resolved target profile.", supports_damage_types = "`supports_damage_types` provides the switch that enables or disables the behavior used to create a `VersionCaps` for a concrete resolved target profile.", supports_chat_types = "`supports_chat_types` provides the switch that enables or disables the behavior used to create a `VersionCaps` for a concrete resolved target profile.", supports_enchantments = "`supports_enchantments` provides the switch that enables or disables the behavior used to create a `VersionCaps` for a concrete resolved target profile.", supports_trim_assets = "`supports_trim_assets` provides the switch that enables or disables the behavior used to create a `VersionCaps` for a concrete resolved target profile.", supports_item_components = "`supports_item_components` provides the switch that enables or disables the behavior used to create a `VersionCaps` for a concrete resolved target profile."),
        returns = "A `VersionCaps` for the concrete resolved target profile.",
        example = "use sand::prelude::*;\n\nfn demonstrate(requested_version: impl Into < String >, is_fallback: bool, supports_dialogs: bool, supports_jukebox_songs: bool, supports_damage_types: bool, supports_chat_types: bool, supports_enchantments: bool, supports_trim_assets: bool, supports_item_components: bool)  {\n    let version_caps = sand::version::VersionCaps::from_profile_flags(requested_version, is_fallback, supports_dialogs, supports_jukebox_songs, supports_damage_types, supports_chat_types, supports_enchantments, supports_trim_assets, supports_item_components);\n}",
    )]
    #[allow(clippy::too_many_arguments)]
    pub fn from_profile_flags(
        requested_version: impl Into<String>,
        is_fallback: bool,
        supports_dialogs: bool,
        supports_jukebox_songs: bool,
        supports_damage_types: bool,
        supports_chat_types: bool,
        supports_enchantments: bool,
        supports_trim_assets: bool,
        supports_item_components: bool,
    ) -> Self {
        Self {
            requested_version: requested_version.into(),
            is_fallback,
            supports_dialogs,
            supports_jukebox_songs,
            supports_damage_types,
            supports_chat_types,
            supports_enchantments,
            supports_trim_assets,
            supports_item_components,
            // Not constructor parameters — see `with_animal_variants` /
            // `with_villager_trades`. Callers that need to express them chain
            // `.with_animal_variants(true)` / `.with_villager_trades(true)`.
            supports_animal_variants: false,
            supports_villager_trades: false,
        }
    }

    /// Version requested by the project that produced these capabilities.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::version::VersionCaps::requested_version",
        module = "sand::version",
        kind = "method",
        summary = "Version requested by the project that produced these capabilities.",
        context = "Version requested by the project that produced these capabilities. This version API lets reusable authoring and tooling make the same capability decisions as Sand's profile-aware component exporter.",
        minecraft = "Capability checks describe the data-driven features accepted by the selected Minecraft Java Edition target before pack output is written.",
        use_when = ["Adapting authored resources or integrations to an explicitly selected Minecraft target"],
        avoid_when = ["Ordinary datapack code can rely on the target selected in sand.toml"],
        returns = "The string value produced to version requested by the project that produced these capabilities.",
        example = "use sand::prelude::*;\n\nfn demonstrate(version_caps_value: &sand::version::VersionCaps)  {\n    let requested_version = version_caps_value.requested_version();\n}",
    )]
    pub fn requested_version(&self) -> &str {
        &self.requested_version
    }

    /// Whether the version resolver used conservative fallback capabilities.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::version::VersionCaps::is_fallback",
        module = "sand::version",
        kind = "method",
        summary = "Whether the version resolver used conservative fallback capabilities.",
        context = "Whether the version resolver used conservative fallback capabilities. This version API lets reusable authoring and tooling make the same capability decisions as Sand's profile-aware component exporter.",
        minecraft = "Capability checks describe the data-driven features accepted by the selected Minecraft Java Edition target before pack output is written.",
        use_when = ["Adapting authored resources or integrations to an explicitly selected Minecraft target"],
        avoid_when = ["Ordinary datapack code can rely on the target selected in sand.toml"],
        returns = "`true` when the documented condition holds to determine whether the version resolver used conservative fallback capabilities; otherwise `false`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(version_caps_value: &sand::version::VersionCaps)  {\n    let is_is_fallback = version_caps_value.is_fallback();\n}",
    )]
    pub fn is_fallback(&self) -> bool {
        self.is_fallback
    }

    /// Compare the requested target with a concrete Minecraft release.
    ///
    /// `latest` resolves to [`LATEST_KNOWN`]. Unknown/fallback targets return
    /// `false`; callers must not infer schema support from a fallback profile.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::version::VersionCaps::is_at_least",
        module = "sand::version",
        kind = "method",
        summary = "Compare the requested target with a concrete Minecraft release.",
        context = "Compare the requested target with a concrete Minecraft release. `latest` resolves to [`LATEST_KNOWN`]. Unknown/fallback targets return `false`; callers must not infer schema support from a fallback profile.",
        minecraft = "Capability checks describe the data-driven features accepted by the selected Minecraft Java Edition target before pack output is written.",
        use_when = ["Adapting authored resources or integrations to an explicitly selected Minecraft target"],
        avoid_when = ["Ordinary datapack code can rely on the target selected in sand.toml"],
        params(major = "`major` is the major used when comparing the requested target with a concrete Minecraft release.", minor = "`minor` is the minor used when comparing the requested target with a concrete Minecraft release.", patch = "`patch` is the patch used when comparing the requested target with a concrete Minecraft release."),
        returns = "`true` when the documented condition holds to compare the requested target with a concrete Minecraft release; otherwise `false`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(version_caps_value: &sand::version::VersionCaps, major: u32, minor: u32, patch: u32)  {\n    let is_is_at_least = version_caps_value.is_at_least(major, minor, patch);\n}",
    )]
    pub fn is_at_least(&self, major: u32, minor: u32, patch: u32) -> bool {
        if self.is_fallback {
            return false;
        }
        let value = if self.requested_version == "latest" {
            LATEST_KNOWN
        } else {
            &self.requested_version
        };
        let mut parts = value.split('.').map(|part| part.parse::<u32>());
        let Some(Ok(actual_major)) = parts.next() else {
            return false;
        };
        let actual_minor = parts.next().transpose().ok().flatten().unwrap_or(0);
        let actual_patch = parts.next().transpose().ok().flatten().unwrap_or(0);
        (actual_major, actual_minor, actual_patch) >= (major, minor, patch)
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
        assert_eq!(ComponentFeature::ItemComponents.name(), "item_components");
        assert_eq!(ComponentFeature::AnimalVariants.name(), "animal_variants");
        assert_eq!(ComponentFeature::VillagerTrades.name(), "villager_trades");
    }

    #[test]
    fn from_flags_respects_individual_values() {
        let caps = VersionCaps::from_flags(true, false, true, false, true, false, true);
        assert!(caps.supports(ComponentFeature::Dialogs));
        assert!(!caps.supports(ComponentFeature::JukeboxSongs));
        assert!(caps.supports(ComponentFeature::DamageTypes));
        assert!(!caps.supports(ComponentFeature::ChatTypes));
        assert!(caps.supports(ComponentFeature::Enchantments));
        assert!(!caps.supports(ComponentFeature::TrimAssets));
        assert!(caps.supports(ComponentFeature::ItemComponents));
        assert_eq!(caps.requested_version(), LATEST_KNOWN);
        assert!(!caps.is_fallback());
    }

    #[test]
    fn with_animal_variants_overrides_without_touching_other_flags() {
        let caps = VersionCaps::from_flags(true, true, true, true, true, true, true)
            .with_animal_variants(true);
        assert!(caps.supports(ComponentFeature::AnimalVariants));
        assert!(caps.supports(ComponentFeature::Dialogs));

        let caps = VersionCaps::all_enabled().with_animal_variants(false);
        assert!(!caps.supports(ComponentFeature::AnimalVariants));
        assert!(caps.supports(ComponentFeature::Dialogs));
    }

    #[test]
    fn with_villager_trades_overrides_without_touching_other_flags() {
        let caps = VersionCaps::from_flags(true, true, true, true, true, true, true)
            .with_villager_trades(true);
        assert!(caps.supports(ComponentFeature::VillagerTrades));
        assert!(caps.supports(ComponentFeature::Dialogs));

        let caps = VersionCaps::all_enabled().with_villager_trades(false);
        assert!(!caps.supports(ComponentFeature::VillagerTrades));
        assert!(caps.supports(ComponentFeature::Dialogs));
    }

    #[test]
    fn profiled_caps_compare_versions_without_guessing_for_fallbacks() {
        let stable = VersionCaps::from_profile_flags(
            "1.21.4", false, false, true, true, true, true, true, true,
        );
        assert!(stable.is_at_least(1, 20, 5));
        assert!(!stable.is_at_least(26, 2, 0));

        let fallback = VersionCaps::all_disabled();
        assert!(!fallback.is_at_least(1, 0, 0));
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

    #[test]
    fn vanilla_reload_workflow_uses_verified_matrix_source() {
        let workflow = include_str!("../../.github/workflows/vanilla-reload.yml");
        assert!(workflow.contains("--bin vanilla-reload-matrix"));
        assert!(workflow.contains("fromJSON(needs.versions.outputs.matrix)"));
        assert!(workflow.contains("actions/upload-artifact@v4"));
        assert!(workflow.contains("target/vanilla-reload/${{ matrix.version }}/latest.log"));
        assert!(!CI_STABLE_CODEGEN_VERSION.is_empty());
        assert!(!LATEST_KNOWN.is_empty());
        assert_eq!(CI_STABLE_JAVA_VERSION, "21");
        assert_eq!(CI_LATEST_JAVA_VERSION, "25");
    }
}
