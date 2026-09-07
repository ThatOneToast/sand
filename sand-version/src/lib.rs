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

/// Java runtime required by the verified vanilla-server validation target.
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

    /// Current baseline profile used by direct command rendering without
    /// project configuration. Exporters should pass the project's resolved profile.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::CommandProfile::unprofiled",
        aliases = ["sand::cmd::CommandProfile::unprofiled", "sand::prelude::cmd::CommandProfile::unprofiled"],
        module = "sand::command",
        kind = "method",
        summary = "Current baseline profile used by direct command rendering without project configuration. Exporters should pass the project's resolved profile.",
        context = "Current baseline profile used by direct command rendering without project configuration. Exporters should pass the project's resolved profile. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `CommandProfile` configured for the current baseline when direct rendering has no project configuration.",
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
    variants(AnimalVariants = "Biome-scoped animal variant registries in verified 26.x schemas.", ChatTypes = "Chat type registries in verified 26.x schemas.", DamageTypes = "Damage type registries in verified 26.x schemas.", Dialogs = "Data-driven dialogs in verified 26.x schemas.", Enchantments = "Enchantment data components in verified 26.x schemas.", ItemComponents = "Structured item components in verified 26.x schemas.", JukeboxSongs = "Jukebox song components in verified 26.x schemas.", TrimAssets = "Armor trim assets in verified 26.x schemas.", VillagerTrades = "Data-driven Villager/Wandering Trader trade registries in verified 26.x schemas."),
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentFeature {
    /// Data-driven dialogs in verified 26.x schemas.
    Dialogs,
    /// Jukebox song components in verified 26.x schemas.
    JukeboxSongs,
    /// Damage type registries in verified 26.x schemas.
    DamageTypes,
    /// Chat type registries in verified 26.x schemas.
    ChatTypes,
    /// Enchantment data components in verified 26.x schemas.
    Enchantments,
    /// Armor trim assets in verified 26.x schemas.
    TrimAssets,
    /// Structured item components in verified 26.x schemas.
    ItemComponents,
    /// Biome-scoped animal variant registries in verified 26.x schemas.
    AnimalVariants,
    /// Data-driven Villager/Wandering Trader trade registries.
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
    /// Used by the unprofiled export path, which targets Sand's current
    /// verified baseline.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::version::VersionCaps::all_enabled",
        module = "sand::version",
        kind = "method",
        summary = "Create a `VersionCaps` where all features are enabled.",
        context = "Create a `VersionCaps` where all features are enabled. Used by the unprofiled export path, which targets Sand's current verified baseline.",
        minecraft = "Used by the unprofiled export path for Sand's current verified baseline.",
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
            requested_version: "unknown".to_string(),
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

    /// Set whether biome-scoped animal variant registries are present in the
    /// exact target schema.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::version::VersionCaps::with_animal_variants",
        module = "sand::version",
        kind = "method",
        summary = "Set whether biome-scoped animal variant registries are present in the exact target schema.",
        context = "This capability is false for conservative unknown-future profiles until their generated schema is verified.",
        minecraft = "Capability checks describe the data-driven features accepted by the selected Minecraft Java Edition target before pack output is written.",
        use_when = ["Adapting authored resources or integrations to an explicitly selected Minecraft target"],
        avoid_when = ["Ordinary datapack code can rely on the target selected in sand.toml"],
        params(value = "`value` indicates whether the exact target schema contains the animal variant registries."),
        returns = "The updated target capabilities.",
        example = "use sand::prelude::*;\n\nfn demonstrate(version_caps_value: sand::version::VersionCaps, value: bool)  {\n    let updated_version_caps = version_caps_value.with_animal_variants(value);\n}",
    )]
    pub fn with_animal_variants(mut self, value: bool) -> Self {
        self.supports_animal_variants = value;
        self
    }

    /// Set whether the data-driven Villager/Wandering Trader trade
    /// registries are supported.
    ///
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::version::VersionCaps::with_villager_trades",
        module = "sand::version",
        kind = "method",
        summary = "Set whether the data-driven Villager/Wandering Trader trade registries are supported.",
        context = "This capability is enabled for verified 26.x profiles and false for conservative unknown-future profiles until their generated schema is verified.",
        minecraft = "Capability checks describe the data-driven features accepted by the selected Minecraft Java Edition target before pack output is written.",
        use_when = ["Adapting authored resources or integrations to an explicitly selected Minecraft target"],
        avoid_when = ["Ordinary datapack code can rely on the target selected in sand.toml"],
        params(value = "`value` selects whether the target schema supports data-driven Villager/Wandering Trader trade registries."),
        returns = "The updated `VersionCaps` value.",
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
    fn capability_sets_fail_closed_for_unknown_schemas() {
        let known = VersionCaps::all_enabled();
        let unknown = VersionCaps::all_disabled();
        for feature in ComponentFeature::ALL {
            assert!(known.supports(*feature), "{feature:?} should be enabled");
            assert!(
                !unknown.supports(*feature),
                "{feature:?} should fail closed"
            );
        }
        assert!(known.is_at_least(26, 1, 0));
        assert!(!unknown.is_at_least(26, 0, 0));
    }

    #[test]
    fn narrow_capabilities_can_be_overridden_without_affecting_others() {
        let caps = VersionCaps::all_enabled()
            .with_animal_variants(false)
            .with_villager_trades(false);
        assert!(!caps.supports(ComponentFeature::AnimalVariants));
        assert!(!caps.supports(ComponentFeature::VillagerTrades));
        assert!(caps.supports(ComponentFeature::Dialogs));
    }

    #[test]
    fn version_anchors_are_exact() {
        assert_eq!(LATEST_KNOWN, "26.2");
        assert_eq!(DEFAULT_CODEGEN_VERSION, LATEST_KNOWN);
        assert_eq!(CI_LATEST_JAVA_VERSION, "25");
    }
}
