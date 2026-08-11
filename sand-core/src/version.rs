//! Minecraft version compatibility layer.
//!
//! Provides a single source of truth for version parsing, pack format lookup,
//! and feature flags across supported 1.x and 26.x Java Edition versions.
//!
//! # Quick start
//! ```
//! use sand_core::version::{MinecraftVersion, VersionFeature, VersionProfile};
//!
//! let v = MinecraftVersion::parse("1.21.4").unwrap();
//! let profile = VersionProfile::resolve(&v).unwrap();
//! assert_eq!(profile.data_pack_format(), 61);
//! assert_eq!(profile.resource_pack_format(), 46);
//! assert!(profile.supports(VersionFeature::ItemComponents));
//! ```

use std::fmt;

use sand_macros::api;
use thiserror::Error;

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors from version parsing or profile resolution.
#[non_exhaustive]
#[derive(Debug, Error, PartialEq, Eq)]
#[api(
    registry = sand_api_contract,
    path = "sand::version::VersionError",
    module = "sand::version",
    summary = "Reports an invalid or unverified Minecraft target version.",
    context = "Version configuration must distinguish malformed input from a syntactically valid release that Sand has not verified yet.",
    minecraft = "Prevents Sand from selecting unsupported pack formats or feature gates for an invalid target release.",
    use_when = ["Handling a version supplied by configuration or a build integration"],
    avoid_when = ["Representing an accepted target version"],
    example = "let version = MinecraftVersion::parse(\"1.21.4\")?;",
    variants(
        ParseError = "Carries text that cannot be parsed as a Minecraft Java version.",
        UnknownVersion = "Reports a parseable version that has no exact verified Sand profile."
    ),
    variant_fields(
        ParseError = ["The original malformed version text."],
        UnknownVersion(requested = "The parseable Minecraft version that lacks an exact verified profile.")
    )
)]
pub enum VersionError {
    /// The version string could not be parsed.
    #[error(
        "Invalid version '{0}': expected examples like '1.19.4', '1.20.6', '1.21.11', '26', '26.2', '26.1.2', or 'latest'"
    )]
    ParseError(String),
    /// The version was parsed but is not in the known table.
    ///
    /// Use [`VersionProfile::resolve`] (which returns a conservative fallback) or
    /// add `pack_format` / `resource_pack_format` overrides to `sand.toml`.
    #[error(
        "Unknown or unverified Minecraft version '{requested}'. Add an explicit `pack_format` override in sand.toml, or use VersionProfile::resolve to accept a conservative fallback for local experimentation."
    )]
    UnknownVersion { requested: MinecraftVersion },
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
/// assert!(a.is_legacy_series());
/// assert!(b.is_26_series());
/// assert!(c.is_latest());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[api(
    registry = sand_api_contract,
    path = "sand::version::MinecraftVersion",
    aliases = ["sand::prelude::MinecraftVersion"],
    module = "sand::version",
    summary = "Represents a parsed Minecraft Java Edition target version.",
    context = "A typed version preserves the distinction between an explicit release and Sand's latest-known token across profile resolution and feature checks.",
    minecraft = "Selects Minecraft's version-dependent datapack formats and feature availability.",
    use_when = ["Resolving a target VersionProfile", "Comparing a target against a typed minimum release"],
    avoid_when = ["Passing an unchecked configuration string through a version-aware API"],
    example = "let target = MinecraftVersion::parse(\"1.21.4\")?;"
)]
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
    /// Accepted formats include `"1.19.4"`, `"1.20.6"`, `"1.21.11"`,
    /// `"26"`, `"26.2"`, `"26.1.2"`, and `"latest"`.
    #[api(
        registry = sand_api_contract,
        path = "sand::version::MinecraftVersion::parse",
        aliases = ["sand::prelude::MinecraftVersion::parse"],
        module = "sand::version",
        summary = "Parses a Minecraft Java Edition release or the latest token.",
        context = "Parsing once at the configuration boundary lets later APIs reject malformed versions without carrying raw strings.",
        minecraft = "Accepts legacy 1.x releases, calendar 26.x releases, and latest for Sand's verified release table.",
        use_when = ["Reading a target version from configuration", "Creating a typed comparison minimum"],
        avoid_when = ["Reusing a VersionProfile that is already resolved"],
        params(s = "The release text, such as 1.21.4, 26.2, or latest."),
        returns = "A validated typed version or a VersionError describing malformed input.",
        example = "let target = MinecraftVersion::parse(\"26.2\")?;"
    )]
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
    #[api(registry = sand_api_contract, path = "sand::version::MinecraftVersion::is_latest", aliases = ["sand::prelude::MinecraftVersion::is_latest"], module = "sand::version", summary = "Checks whether this value is Sand's latest token.", context = "The latest token resolves through the current verified release table rather than storing a concrete release in author configuration.", minecraft = "Latest selects Sand's newest verified Minecraft target during profile resolution.", use_when = ["Preserving a caller's latest-versus-explicit choice"], avoid_when = ["Determining whether a concrete release supports a feature"], returns = "True when this value was parsed from latest.", example = "assert!(MinecraftVersion::parse(\"latest\")?.is_latest());")]
    pub fn is_latest(&self) -> bool {
        matches!(self.kind, VersionKind::Latest)
    }

    /// Returns `true` for the legacy `1.x` version series (e.g. `1.21.4`).
    #[api(registry = sand_api_contract, path = "sand::version::MinecraftVersion::is_legacy_series", aliases = ["sand::prelude::MinecraftVersion::is_legacy_series"], module = "sand::version", summary = "Checks whether this is a legacy 1.x Minecraft release.", context = "Sand supports both the historical 1.x release names and Mojang's newer calendar series.", minecraft = "Legacy releases use the 1.x Java Edition version scheme, such as 1.21.4.", use_when = ["Selecting logic that intentionally differs between legacy and calendar release families"], avoid_when = ["Checking an individual Minecraft capability"], returns = "True for an explicit 1.x release.", example = "assert!(MinecraftVersion::parse(\"1.21.4\")?.is_legacy_series());")]
    pub fn is_legacy_series(&self) -> bool {
        matches!(self.kind, VersionKind::Specific { major: 1, .. })
    }

    /// Returns `true` for the new `26.x` calendar series.
    #[api(registry = sand_api_contract, path = "sand::version::MinecraftVersion::is_26_series", aliases = ["sand::prelude::MinecraftVersion::is_26_series"], module = "sand::version", summary = "Checks whether this is a calendar-series 26.x Minecraft release.", context = "The calendar series succeeded the legacy 1.x naming scheme and can require distinct authoring behavior.", minecraft = "Calendar releases use names such as 26.1 and 26.2.", use_when = ["Selecting behavior intentionally specific to Mojang's calendar release series"], avoid_when = ["Checking a feature represented by VersionFeature"], returns = "True for an explicit 26.x release.", example = "assert!(MinecraftVersion::parse(\"26.2\")?.is_26_series());")]
    pub fn is_26_series(&self) -> bool {
        matches!(self.kind, VersionKind::Specific { major: 26, .. })
    }

    /// Return major, minor, patch components if this is a specific version.
    #[api(registry = sand_api_contract, path = "sand::version::MinecraftVersion::components", aliases = ["sand::prelude::MinecraftVersion::components"], module = "sand::version", summary = "Returns numeric components for an explicit release.", context = "The latest token is intentionally not a fixed numeric version until profile resolution chooses Sand's current verified anchor.", minecraft = "Minecraft releases are compared as major, minor, and patch numbers when they are explicit.", use_when = ["Displaying or adapting an explicit release number"], avoid_when = ["Comparing targets; use is_at_least instead"], returns = "The major, minor, and patch components, or None for latest.", example = "assert_eq!(MinecraftVersion::parse(\"1.21.4\")?.components(), Some((1, 21, 4)));" )]
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

    /// Returns `true` when this version meets or exceeds a typed minimum.
    ///
    /// `latest` always satisfies a historical minimum (it resolves to the
    /// newest known version). Calendar `26.x` versions compare greater than
    /// any legacy `1.x` minimum by ordinary numeric ordering, since Mojang's
    /// calendar series is always newer than the legacy series it succeeded.
    ///
    /// # Examples
    /// ```
    /// use sand_core::version::MinecraftVersion;
    ///
    /// let v = MinecraftVersion::parse("1.21.4").unwrap();
    /// assert!(v.is_at_least(&MinecraftVersion::parse("1.20.2").unwrap()));
    /// assert!(!v.is_at_least(&MinecraftVersion::parse("1.21.5").unwrap()));
    ///
    /// let v26 = MinecraftVersion::parse("26.1").unwrap();
    /// assert!(v26.is_at_least(&MinecraftVersion::parse("1.21.2").unwrap()));
    ///
    /// assert!(MinecraftVersion::parse("latest").unwrap().is_at_least(
    ///     &MinecraftVersion::parse("1.99").unwrap()
    /// ));
    /// ```
    #[api(
        registry = sand_api_contract,
        path = "sand::version::MinecraftVersion::is_at_least",
        aliases = ["sand::prelude::MinecraftVersion::is_at_least"],
        module = "sand::version",
        summary = "Checks whether this target meets a typed minimum Minecraft release.",
        context = "Typed comparison avoids scattering numeric release triples and gives latest a deliberate verified-anchor meaning.",
        minecraft = "Compares Minecraft Java release ordering; latest resolves to Sand's newest verified release for the comparison.",
        use_when = ["Gating a narrow compatibility behavior on a release boundary"],
        avoid_when = ["Checking a named capability represented by VersionFeature"],
        params(minimum = "The validated minimum Minecraft release to require."),
        returns = "True when this target is at least the supplied minimum.",
        example = "assert!(MinecraftVersion::parse(\"26.2\")?.is_at_least(&MinecraftVersion::parse(\"1.21.4\")?));"
    )]
    pub fn is_at_least(&self, minimum: &Self) -> bool {
        fn resolved(version: &MinecraftVersion) -> (u32, u32, u32) {
            version.components().unwrap_or_else(|| {
                MinecraftVersion::parse(LATEST_KNOWN)
                    .expect("LATEST_KNOWN must be a valid Minecraft version")
                    .components()
                    .expect("LATEST_KNOWN must be specific")
            })
        }
        resolved(self) >= resolved(minimum)
    }

    pub(crate) fn is_at_least_components(&self, major: u32, minor: u32, patch: u32) -> bool {
        self.is_at_least(
            &Self::parse(&format!("{major}.{minor}.{patch}"))
                .expect("component arguments always form a valid Minecraft version"),
        )
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
/// use sand_core::version::{MinecraftVersion, VersionFeature, VersionProfile};
///
/// // Known 1.21 version → exact profile
/// let v = MinecraftVersion::parse("1.21.4").unwrap();
/// let p = VersionProfile::resolve(&v).unwrap();
/// assert_eq!(p.data_pack_format(), 61);
/// assert!(!p.is_fallback());
///
/// // Known 26.x version → exact profile with full feature support
/// let v = MinecraftVersion::parse("26.1").unwrap();
/// let p = VersionProfile::resolve(&v).unwrap();
/// assert!(p.supports(VersionFeature::CalendarSeries26));
/// assert!(!p.is_fallback(), "26.1 is a verified, mapped version");
/// assert_eq!(p.data_pack_format(), 101);
/// assert!(p.supports(VersionFeature::ItemComponents));
///
/// // Unknown future 26.x → conservative fallback; feature flags false
/// let v = MinecraftVersion::parse("26.99").unwrap();
/// let p = VersionProfile::resolve(&v).unwrap();
/// assert!(p.is_fallback(), "26.99 is beyond the known table");
/// assert!(!p.supports(VersionFeature::Dialogs));
/// ```
#[derive(Debug, Clone)]
#[api(
    registry = sand_api_contract,
    path = "sand::version::VersionProfile",
    aliases = ["sand::prelude::VersionProfile"],
    module = "sand::version",
    summary = "Resolves a Minecraft target into pack formats and typed capabilities.",
    context = "A profile is the single immutable compatibility decision shared by resource generation, export validation, and author-level feature gates.",
    minecraft = "Maps a Minecraft Java release to its datapack/resource-pack formats and availability of versioned vanilla features.",
    use_when = ["Resolving a target release before authoring version-aware content", "Checking a VersionFeature before emitting optional content"],
    avoid_when = ["Constructing contradictory feature flags or pack formats by hand"],
    example = "let profile = VersionProfile::resolve(&MinecraftVersion::parse(\"1.21.4\")?)?;"
)]
pub struct VersionProfile {
    /// The version that was requested.
    requested: MinecraftVersion,
    /// Human-readable resolved name (e.g. `"1.21.4"` or `"26.1 (26-series fallback)"`).
    resolved_name: String,
    /// Data pack format number for `pack.mcmeta`.
    data_pack_format: u32,
    /// Resource pack format number for `pack.mcmeta`.
    resource_pack_format: u32,
    /// Whether this version supports item components (data components, 1.20.5+).
    supports_item_components: bool,
    /// Whether this version supports `data modify` components (1.20.2+).
    supports_data_components: bool,
    /// Whether this is the new 26.x calendar-versioned series.
    supports_26_series: bool,
    /// Whether this version supports data-driven dialogs (1.21.6+ / 26.x).
    supports_dialogs: bool,
    /// Whether this version supports function macros — `$()` syntax (1.20.2+).
    supports_function_macros: bool,
    /// Whether this version supports predicates (always true in 1.15+, our minimum).
    supports_predicates: bool,
    /// Whether this version supports resource pack overlays (1.20.2+).
    supports_resource_pack_overlays: bool,
    /// Whether this version supports trim assets — armor trims (1.19.4+).
    supports_trim_assets: bool,
    /// Whether this version supports jukebox song components (1.21+).
    supports_jukebox_songs: bool,
    /// Whether this version supports damage type registries (1.19.4+).
    supports_damage_types: bool,
    /// Whether this version supports chat type registries (1.19+).
    supports_chat_types: bool,
    /// Whether this version supports enchantment data components (1.21+).
    supports_enchantments: bool,
    /// Whether this version supports biome-scoped animal variant registries —
    /// `chicken_variant`, `cow_variant`, `pig_variant` (1.21.5+).
    supports_animal_variants: bool,
    /// Whether this version supports the data-driven Villager/Wandering
    /// Trader trade registries — `villager_trade` and `trade_set` (26.1+).
    /// Not backported to the legacy `1.21.x` series.
    supports_villager_trades: bool,
    /// When `true` the profile was resolved via a conservative fallback because
    /// the exact version was not in the known table. Users should verify and
    /// may override `pack_format` in `sand.toml`.
    is_fallback: bool,
}

/// The latest version this table was last verified against.
#[api(
    registry = sand_api_contract,
    path = "sand::version::LATEST_KNOWN",
    module = "sand::version",
    summary = "Names the newest Minecraft release verified by Sand's version table.",
    context = "The anchor resolves the latest token and gives author-facing diagnostics a stable verified target.",
    minecraft = "Identifies the release whose pack formats and feature matrix Sand currently verifies as latest.",
    use_when = ["Displaying Sand's verified release anchor", "Resolving the latest token through VersionProfile"],
    avoid_when = ["Choosing Sand's build-time code generator target"],
    example = "assert_eq!(sand::version::LATEST_KNOWN, \"26.2\");"
)]
pub const LATEST_KNOWN: &str = sand_version::LATEST_KNOWN;

#[cfg(test)]
const DEFAULT_CODEGEN_VERSION: &str = sand_version::DEFAULT_CODEGEN_VERSION;

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
/// assert_eq!(meta.pack_format(), 61);
/// assert!(!meta.is_fallback());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[api(
    registry = sand_api_contract,
    path = "sand::version::PackMetadata",
    module = "sand::version",
    summary = "Carries the pack format selected for one Minecraft pack root.",
    context = "Datapacks and resource packs use distinct pack formats, so metadata is obtained from a resolved VersionProfile rather than constructed by hand.",
    minecraft = "Supplies the pack_format value written to a pack.mcmeta file.",
    use_when = ["Writing a datapack or resource-pack metadata file for a resolved target"],
    avoid_when = ["Guessing a pack format from a raw release string"],
    example = "let metadata = profile.datapack_metadata();"
)]
pub struct PackMetadata {
    /// The `pack.pack_format` value to write to `pack.mcmeta`.
    pack_format: u32,
    /// `true` if this metadata was resolved from a conservative fallback because
    /// the exact version was not in the known table.  The caller should warn
    /// the user and accept an override from `sand.toml`.
    is_fallback: bool,
}

impl PackMetadata {
    /// The `pack_format` value to write to this pack's `pack.mcmeta`.
    #[api(registry = sand_api_contract, path = "sand::version::PackMetadata::pack_format", module = "sand::version", summary = "Returns the selected pack.mcmeta format number.", context = "The format is kept with its fallback status so export code cannot accidentally separate them.", minecraft = "Writes the integer Minecraft reads from pack.pack_format.", use_when = ["Serializing the pack section of pack.mcmeta"], avoid_when = ["Selecting a format without resolving a VersionProfile"], returns = "The exact pack format number for this pack root.", example = "assert_eq!(profile.datapack_metadata().pack_format(), 61);")]
    pub fn pack_format(&self) -> u32 {
        self.pack_format
    }

    /// Whether this metadata came from a conservative unknown-version fallback.
    #[api(registry = sand_api_contract, path = "sand::version::PackMetadata::is_fallback", module = "sand::version", summary = "Reports whether this format came from a conservative fallback profile.", context = "Unknown Minecraft releases intentionally receive conservative metadata that exporters should surface to the author.", minecraft = "Fallback formats are based on Sand's known table rather than a verified target release.", use_when = ["Warning before exporting an unverified Minecraft target"], avoid_when = ["Treating an unknown version as fully supported"], returns = "True when the format was not resolved from an exact verified profile.", example = "if profile.datapack_metadata().is_fallback() { /* warn */ }")]
    pub fn is_fallback(&self) -> bool {
        self.is_fallback
    }
}

/// A typed Minecraft capability checked against a [`VersionProfile`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[api(
    registry = sand_api_contract,
    path = "sand::version::VersionFeature",
    aliases = ["sand::prelude::VersionFeature"],
    module = "sand::version",
    summary = "Names a Minecraft capability that can be checked against a target profile.",
    context = "The enum makes every supported capability explicit and prevents a misspelled string gate from silently disabling content.",
    minecraft = "Each variant corresponds to a vanilla feature whose availability changes across Minecraft releases.",
    use_when = ["Gating authored content on a documented Minecraft capability", "Reporting why a target release cannot emit a resource"],
    avoid_when = ["Comparing arbitrary release numbers; use MinecraftVersion for that"],
    example = "if profile.supports(VersionFeature::Dialogs) { /* emit dialogs */ }",
    variants(
        ItemComponents = "The 1.20.5+ structured item-component system.",
        DataComponents = "The data command support for structured item components.",
        CalendarSeries26 = "Mojang's 26.x calendar release series.",
        Dialogs = "Data-driven Minecraft dialogs introduced in 1.21.6 and 26.x.",
        FunctionMacros = "Function macro substitution using $() syntax.",
        Predicates = "Reusable predicate JSON resources supported by Sand's target range.",
        ResourcePackOverlays = "Resource-pack overlay declarations.",
        TrimAssets = "Armor trim material and pattern assets.",
        JukeboxSongs = "Data-driven jukebox song components.",
        DamageTypes = "Data-driven damage-type registries.",
        ChatTypes = "Data-driven chat-type registries.",
        Enchantments = "Data-driven enchantment components.",
        AnimalVariants = "Biome-scoped animal variant registries.",
        VillagerTrades = "Data-driven villager-trade and trade-set registries."
    )
)]
pub enum VersionFeature {
    ItemComponents,
    DataComponents,
    CalendarSeries26,
    Dialogs,
    FunctionMacros,
    Predicates,
    ResourcePackOverlays,
    TrimAssets,
    JukeboxSongs,
    DamageTypes,
    ChatTypes,
    Enchantments,
    AnimalVariants,
    VillagerTrades,
}

impl VersionProfile {
    /// Resolve a [`MinecraftVersion`] into a [`VersionProfile`].
    ///
    /// Returns `Ok(profile)` for any parseable version. Unknown future versions
    /// receive a conservative fallback profile (see [`VersionProfile::is_fallback`]).
    #[api(registry = sand_api_contract, path = "sand::version::VersionProfile::resolve", aliases = ["sand::prelude::VersionProfile::resolve"], module = "sand::version", summary = "Resolves any valid Minecraft version to Sand's compatibility profile.", context = "Known versions use exact table entries; future but parseable releases receive conservative metadata so authors can deliberately choose whether to proceed.", minecraft = "Selects pack formats and feature gates for the target Minecraft release.", use_when = ["Resolving a project target that may be newer than Sand's verified table"], avoid_when = ["A release must be exact and verified; use resolve_strict"], params(version = "The validated Minecraft release or latest token to resolve."), returns = "An exact or conservative VersionProfile for the target.", example = "let profile = VersionProfile::resolve(&MinecraftVersion::parse(\"26.2\")?)?;")]
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
            supports_animal_variants: caps.animal_variants,
            supports_villager_trades: caps.villager_trades,
            is_fallback: caps.is_fallback,
        })
    }

    /// The parsed version requested by the project configuration.
    #[api(registry = sand_api_contract, path = "sand::version::VersionProfile::requested", aliases = ["sand::prelude::VersionProfile::requested"], module = "sand::version", summary = "Returns the version originally requested for this profile.", context = "The requested value preserves latest instead of replacing the author's configuration with the resolved anchor.", minecraft = "Latest can resolve to a verified release while still remaining a distinct configuration choice.", use_when = ["Displaying or retaining the project's version choice"], avoid_when = ["Needing the resolved table label; use resolved_name"], returns = "The parsed requested Minecraft version.", example = "assert!(profile.requested().is_latest());")]
    pub fn requested(&self) -> &MinecraftVersion {
        &self.requested
    }

    /// Sand's exact table entry or fallback label selected for the target.
    #[api(registry = sand_api_contract, path = "sand::version::VersionProfile::resolved_name", aliases = ["sand::prelude::VersionProfile::resolved_name"], module = "sand::version", summary = "Returns Sand's exact resolved profile label.", context = "The label explains whether latest or an unknown future release was mapped through the version table.", minecraft = "Identifies the Minecraft profile whose pack formats and feature gates are in effect.", use_when = ["Rendering a build diagnostic or export report"], avoid_when = ["Comparing releases; use typed MinecraftVersion"], returns = "A human-readable exact or fallback profile name.", example = "println!(\"target: {}\", profile.resolved_name());")]
    pub fn resolved_name(&self) -> &str {
        &self.resolved_name
    }

    /// The data-pack `pack_format` required by this target.
    #[api(registry = sand_api_contract, path = "sand::version::VersionProfile::data_pack_format", aliases = ["sand::prelude::VersionProfile::data_pack_format"], module = "sand::version", summary = "Returns the datapack pack.mcmeta format for this target.", context = "The profile owns format selection so datapack metadata cannot drift from capability checks.", minecraft = "Supplies the pack_format accepted for data packs by the target Minecraft release.", use_when = ["Inspecting datapack metadata before export"], avoid_when = ["Writing a resource-pack format; use resource_pack_format"], returns = "The target datapack format number.", example = "let format = profile.data_pack_format();")]
    pub fn data_pack_format(&self) -> u32 {
        self.data_pack_format
    }

    /// The resource-pack `pack_format` required by this target.
    #[api(registry = sand_api_contract, path = "sand::version::VersionProfile::resource_pack_format", aliases = ["sand::prelude::VersionProfile::resource_pack_format"], module = "sand::version", summary = "Returns the resource-pack pack.mcmeta format for this target.", context = "Resource packs have a separately versioned format from datapacks, but both are resolved together.", minecraft = "Supplies the pack_format accepted for resource packs by the target Minecraft release.", use_when = ["Inspecting resource-pack metadata before export"], avoid_when = ["Writing a datapack format; use data_pack_format"], returns = "The target resource-pack format number.", example = "let format = profile.resource_pack_format();")]
    pub fn resource_pack_format(&self) -> u32 {
        self.resource_pack_format
    }

    /// Whether Sand used a conservative profile because the exact release is unknown.
    #[api(registry = sand_api_contract, path = "sand::version::VersionProfile::is_fallback", aliases = ["sand::prelude::VersionProfile::is_fallback"], module = "sand::version", summary = "Reports whether this profile is conservative rather than exact.", context = "Sand never guesses new Minecraft capabilities: unverified releases use a fallback profile that callers can surface or reject.", minecraft = "Fallback targets do not claim availability for version-sensitive vanilla content.", use_when = ["Warning about or rejecting an unverified target"], avoid_when = ["Treating a syntactically valid future release as fully supported"], returns = "True when the requested release has no exact verified profile.", example = "if profile.is_fallback() { /* require an explicit override */ }")]
    pub fn is_fallback(&self) -> bool {
        self.is_fallback
    }

    /// Returns whether this target supports one typed Minecraft capability.
    #[api(registry = sand_api_contract, path = "sand::version::VersionProfile::supports", aliases = ["sand::prelude::VersionProfile::supports"], module = "sand::version", summary = "Checks one typed Minecraft capability for this target.", context = "A single typed query keeps feature gating complete and avoids raw string keys that silently misspell to false.", minecraft = "Reports whether the target release supports the requested vanilla feature family.", use_when = ["Gating optional authored content such as dialogs or resource-pack overlays"], avoid_when = ["Comparing an arbitrary release threshold; use MinecraftVersion::is_at_least"], params(feature = "The explicit Minecraft capability to check."), returns = "True when the resolved target supports that capability.", example = "assert!(profile.supports(VersionFeature::ItemComponents));")]
    pub fn supports(&self, feature: VersionFeature) -> bool {
        match feature {
            VersionFeature::ItemComponents => self.supports_item_components,
            VersionFeature::DataComponents => self.supports_data_components,
            VersionFeature::CalendarSeries26 => self.supports_26_series,
            VersionFeature::Dialogs => self.supports_dialogs,
            VersionFeature::FunctionMacros => self.supports_function_macros,
            VersionFeature::Predicates => self.supports_predicates,
            VersionFeature::ResourcePackOverlays => self.supports_resource_pack_overlays,
            VersionFeature::TrimAssets => self.supports_trim_assets,
            VersionFeature::JukeboxSongs => self.supports_jukebox_songs,
            VersionFeature::DamageTypes => self.supports_damage_types,
            VersionFeature::ChatTypes => self.supports_chat_types,
            VersionFeature::Enchantments => self.supports_enchantments,
            VersionFeature::AnimalVariants => self.supports_animal_variants,
            VersionFeature::VillagerTrades => self.supports_villager_trades,
        }
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
    #[api(registry = sand_api_contract, path = "sand::version::VersionProfile::resolve_strict", aliases = ["sand::prelude::VersionProfile::resolve_strict"], module = "sand::version", summary = "Resolves only a Minecraft version with an exact verified Sand profile.", context = "Release and CI workflows often need to reject unknown future releases rather than emitting conservative metadata.", minecraft = "Requires an exact table entry for the target Minecraft release and its formats/features.", use_when = ["Validating a release build or CI target"], avoid_when = ["Local experimentation where a conservative fallback is intentional"], params(version = "The validated Minecraft version that must have an exact profile."), returns = "An exact VersionProfile or UnknownVersion.", example = "let profile = VersionProfile::resolve_strict(&MinecraftVersion::parse(\"26.2\")?)?;")]
    pub fn resolve_strict(version: &MinecraftVersion) -> Result<Self, VersionError> {
        let profile = Self::resolve(version)?;
        if profile.is_fallback {
            return Err(VersionError::UnknownVersion {
                requested: version.clone(),
            });
        }
        Ok(profile)
    }

    /// Return pack metadata for a datapack using this version profile.
    ///
    /// The returned value contains the exact `pack_format` to write to `pack.mcmeta`.
    /// When `is_fallback` is `true`, both formats are derived from the latest known
    /// version and the caller should warn that the output may not be validated.
    #[api(registry = sand_api_contract, path = "sand::version::VersionProfile::datapack_metadata", aliases = ["sand::prelude::VersionProfile::datapack_metadata"], module = "sand::version", summary = "Builds metadata for a target datapack root.", context = "The metadata carries both the resolved format and its fallback status as one immutable value.", minecraft = "Produces the pack_format Minecraft reads from a datapack's pack.mcmeta.", use_when = ["Writing datapack metadata during export"], avoid_when = ["Writing resource-pack metadata; use resourcepack_metadata"], returns = "The selected datapack PackMetadata.", example = "let metadata = profile.datapack_metadata();")]
    pub fn datapack_metadata(&self) -> PackMetadata {
        PackMetadata {
            pack_format: self.data_pack_format,
            is_fallback: self.is_fallback,
        }
    }

    /// Return pack metadata for a resource pack using this version profile.
    #[api(registry = sand_api_contract, path = "sand::version::VersionProfile::resourcepack_metadata", aliases = ["sand::prelude::VersionProfile::resourcepack_metadata"], module = "sand::version", summary = "Builds metadata for a target resource-pack root.", context = "Resource packs use their own format sequence, selected by the same target profile as the datapack.", minecraft = "Produces the pack_format Minecraft reads from a resource pack's pack.mcmeta.", use_when = ["Writing resource-pack metadata during export"], avoid_when = ["Writing datapack metadata; use datapack_metadata"], returns = "The selected resource-pack PackMetadata.", example = "let metadata = profile.resourcepack_metadata();")]
    pub fn resourcepack_metadata(&self) -> PackMetadata {
        PackMetadata {
            pack_format: self.resource_pack_format,
            is_fallback: self.is_fallback,
        }
    }

    /// Return the cycle-safe capability set for this profile.
    ///
    /// The [`sand_version::VersionCaps`] can be passed to `sand-components`
    /// (which cannot depend on `sand-core`) for version-aware component gating.
    pub(crate) fn caps(&self) -> sand_version::VersionCaps {
        sand_version::VersionCaps::from_profile_flags(
            self.requested.to_string(),
            self.is_fallback,
            self.supports_dialogs,
            self.supports_jukebox_songs,
            self.supports_damage_types,
            self.supports_chat_types,
            self.supports_enchantments,
            self.supports_trim_assets,
            self.supports_item_components,
        )
        .with_animal_variants(self.supports_animal_variants)
        .with_villager_trades(self.supports_villager_trades)
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
    animal_variants: bool,
    villager_trades: bool,
    is_fallback: bool,
}

impl Default for VersionCaps {
    /// All-features-enabled baseline used as a spread target by known-version arms.
    ///
    /// Do NOT use this as the fallback for unknown versions — use
    /// [`VersionCaps::conservative`] instead.
    ///
    /// `villager_trades` defaults to `false` here (unlike every other flag):
    /// the registry is new in the 26.x calendar series and was never
    /// backported to any legacy `1.21.x` release, so every legacy-series arm
    /// would otherwise need an explicit `villager_trades: false` override.
    /// Only the `26.1`/`26.2` arms opt in explicitly.
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
            animal_variants: true,
            villager_trades: false,
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
            data_fmt: 107,
            res_fmt: 88,
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
            animal_variants: false,
            villager_trades: false,
            is_fallback: true,
        }
    }
}

/// Look up version capabilities from (major, minor, patch).
///
/// Pack format numbers sourced from <https://minecraft.wiki/w/Pack_format>.
fn lookup(major: u32, minor: u32, patch: u32) -> VersionCaps {
    match (major, minor, patch) {
        // ════════════════════════════════════════════════════════════════════
        // 26.x calendar series  (2026+, Minecraft's new versioning scheme)
        // ════════════════════════════════════════════════════════════════════

        // ── 26.2 / 26.2.0 — data 107, resource 88 ────────────────────────
        (26, 2, 0) => VersionCaps {
            data_fmt: 107,
            res_fmt: 88,
            dialogs: true,
            villager_trades: true,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 26.1 through 26.1.2 — data 101, resource 84 ──────────────────
        (26, 1, 0..=2) => VersionCaps {
            data_fmt: 101,
            res_fmt: 84,
            dialogs: true,
            villager_trades: true,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 26.x unknown minor — conservative; reject via resolve_strict ──
        (26, _, _) => VersionCaps::conservative(),

        // ════════════════════════════════════════════════════════════════════
        // 1.21.x series
        // ════════════════════════════════════════════════════════════════════

        // ── 1.21.11 — data 94, resource 75 ───────────────────────────────
        (1, 21, 11) => VersionCaps {
            data_fmt: 94,
            res_fmt: 75,
            dialogs: true,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.21.9-10 — data 88, resource 69 ────────────────────────────
        (1, 21, 9..=10) => VersionCaps {
            data_fmt: 88,
            res_fmt: 69,
            dialogs: true,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.21.7-8 — data 81, resource 64 ─────────────────────────────
        (1, 21, 7..=8) => VersionCaps {
            data_fmt: 81,
            res_fmt: 64,
            dialogs: true,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.21.6 — dialogs introduced; data 80, resource 63 ────────────
        (1, 21, 6) => VersionCaps {
            data_fmt: 80,
            res_fmt: 63,
            dialogs: true,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.21.5 — data 71, resource 55 ────────────────────────────────
        (1, 21, 5) => VersionCaps {
            data_fmt: 71,
            res_fmt: 55,
            dialogs: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.21.4 — data 61, resource 46 ────────────────────────────────
        (1, 21, 4) => VersionCaps {
            data_fmt: 61,
            res_fmt: 46,
            dialogs: false,
            animal_variants: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.21.2-3 — data 57, resource 42 ─────────────────────────────
        (1, 21, 2..=3) => VersionCaps {
            data_fmt: 57,
            res_fmt: 42,
            dialogs: false,
            animal_variants: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.21.0-1 — data 48, resource 34 ─────────────────────────────
        (1, 21, 0..=1) => VersionCaps {
            data_fmt: 48,
            res_fmt: 34,
            dialogs: false,
            animal_variants: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── unknown future 1.21.x — keep latest known 1.21 pack formats,
        //    but use conservative capabilities; reject via resolve_strict ─
        (1, 21, _) => VersionCaps {
            data_fmt: 94,
            res_fmt: 75,
            ..VersionCaps::conservative()
        },

        // ════════════════════════════════════════════════════════════════════
        // 1.20.x series
        // ════════════════════════════════════════════════════════════════════

        // ── 1.20.5-6 — data 41, resource 32 ─────────────────────────────
        (1, 20, 5..=6) => VersionCaps {
            data_fmt: 41,
            res_fmt: 32,
            dialogs: false,
            jukebox_songs: false,
            enchantments: false,
            animal_variants: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.20.3-4 — data 26, resource 22 ─────────────────────────────
        (1, 20, 3..=4) => VersionCaps {
            data_fmt: 26,
            res_fmt: 22,
            item_components: false,
            dialogs: false,
            jukebox_songs: false,
            enchantments: false,
            animal_variants: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.20.2 — data 18, resource 18 ────────────────────────────────
        (1, 20, 2) => VersionCaps {
            data_fmt: 18,
            res_fmt: 18,
            item_components: false,
            dialogs: false,
            jukebox_songs: false,
            enchantments: false,
            animal_variants: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.20.0-1 — data 15, resource 15 ─────────────────────────────
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
            animal_variants: false,
            is_fallback: false,
            ..VersionCaps::default()
        },

        // ════════════════════════════════════════════════════════════════════
        // 1.19.x series
        // ════════════════════════════════════════════════════════════════════

        // ── 1.19.4 — data 12, resource 13 ────────────────────────────────
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
            animal_variants: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.19.0-3 — data 10, resource 12 ─────────────────────────────
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
            animal_variants: false,
            is_fallback: false,
            ..VersionCaps::default()
        },

        // ════════════════════════════════════════════════════════════════════
        // 1.18.x series
        // ════════════════════════════════════════════════════════════════════

        // ── 1.18.2 — data 9, resource 8 ──────────────────────────────────
        (1, 18, 2) => VersionCaps {
            data_fmt: 9,
            res_fmt: 8,
            item_components: false,
            data_components: false,
            dialogs: false,
            function_macros: false,
            resource_pack_overlays: false,
            trim_assets: false,
            jukebox_songs: false,
            enchantments: false,
            damage_types: false,
            chat_types: false,
            animal_variants: false,
            is_fallback: false,
            ..VersionCaps::default()
        },
        // ── 1.18.0-1 — data 8, resource 8 ───────────────────────────────
        (1, 18, 0..=1) => VersionCaps {
            data_fmt: 8,
            res_fmt: 8,
            item_components: false,
            data_components: false,
            dialogs: false,
            function_macros: false,
            resource_pack_overlays: false,
            trim_assets: false,
            jukebox_songs: false,
            enchantments: false,
            damage_types: false,
            chat_types: false,
            animal_variants: false,
            is_fallback: false,
            ..VersionCaps::default()
        },

        // ── future 1.x > 1.21 / anything unknown — conservative fallback ─
        _ => VersionCaps::conservative(),
    }
}

// ── Export-time version resolution (#147) ─────────────────────────────────────

/// Resolved version information for the export-time component validation path.
///
/// Produced by [`resolve_export_caps`] from a `sand.toml` `mc_version` string.
/// The [`sand_version::VersionCaps`] field is consumed by `try_export_components_for_version`
/// to gate version-sensitive components.
#[derive(Debug, Clone)]
pub(crate) struct ResolvedExportCaps {
    /// The resolved version string (e.g. `"1.21.4"` or `"26.2"`).
    pub(crate) version: String,
    /// Whether the profile is a conservative fallback (not an exact match).
    pub(crate) is_fallback: bool,
    /// The cycle-safe capability set for component gating.
    pub(crate) caps: sand_version::VersionCaps,
}

/// Resolve a `sand.toml` `mc_version` string into export-time capability info.
///
/// `"latest"` resolves to the bundled [`LATEST_KNOWN`] anchor. Unknown but
/// syntactically valid versions produce a conservative fallback: all feature
/// flags `false`, `is_fallback = true`. Malformed versions return
/// [`crate::error::SandError::InvalidVersion`] rather than silently selecting a
/// fallback. This means version-gated components (dialogs, jukebox songs, etc.)
/// are rejected for fallback/unknown targets unless the user explicitly targets
/// a known exact version or `"latest"`.
///
/// This function is the single resolution point for the export subprocess —
/// it is called by the generated `__sand_export` entrypoint.
pub(crate) fn resolve_export_caps(mc_version: &str) -> crate::error::Result<ResolvedExportCaps> {
    let resolved_version = if mc_version == "latest" {
        LATEST_KNOWN.to_string()
    } else {
        mc_version.to_string()
    };

    let version = MinecraftVersion::parse(&resolved_version)
        .map_err(|_| crate::error::SandError::InvalidVersion(mc_version.to_string()))?;
    match VersionProfile::resolve(&version) {
        Ok(profile) => Ok(ResolvedExportCaps {
            version: profile.resolved_name.clone(),
            is_fallback: profile.is_fallback,
            caps: profile.caps(),
        }),
        Err(_) => Ok(ResolvedExportCaps {
            version: resolved_version,
            is_fallback: true,
            caps: sand_version::VersionCaps::all_disabled(),
        }),
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
        assert!(v.is_legacy_series());
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
        assert!(!v.is_legacy_series());
    }

    #[test]
    fn latest_known_uses_shared_version_anchor() {
        assert_eq!(LATEST_KNOWN, sand_version::LATEST_KNOWN);
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
    fn resolve_121_11_known() {
        let v = MinecraftVersion::parse("1.21.11").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert_eq!(p.data_pack_format, 94);
        assert_eq!(p.resource_pack_format, 75);
        assert!(!p.is_fallback);
        assert!(p.supports(VersionFeature::Dialogs));
    }

    #[test]
    fn resolve_26_1_known() {
        let v = MinecraftVersion::parse("26.1").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(p.supports_26_series);
        assert!(!p.is_fallback, "26.1 is an explicitly mapped version");
        assert_eq!(p.data_pack_format, 101);
        assert_eq!(p.resource_pack_format, 84);
        assert!(p.supports(VersionFeature::Dialogs), "26.1 supports dialogs");
        assert!(p.supports_item_components, "26.1 supports item components");
    }

    #[test]
    fn resolve_26_2_known() {
        let v = MinecraftVersion::parse("26.2").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(p.supports_26_series);
        assert!(!p.is_fallback, "26.2 is an explicitly mapped version");
        assert_eq!(p.data_pack_format, 107);
        assert_eq!(p.resource_pack_format, 88);
        assert!(p.supports(VersionFeature::Dialogs));
    }

    #[test]
    fn resolve_26_unknown_future() {
        let v = MinecraftVersion::parse("26.99").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(p.supports_26_series);
        assert!(p.is_fallback, "26.99 is beyond the known table");
        assert!(
            !p.supports(VersionFeature::Dialogs),
            "unverified version must not claim dialog support"
        );
    }

    #[test]
    fn resolve_latest() {
        let v = MinecraftVersion::parse("latest").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(p.resolved_name.contains("latest"));
        // LATEST_KNOWN = "26.2": data 107, resource 88
        assert_eq!(p.data_pack_format, 107);
        assert_eq!(p.resource_pack_format, 88);
        assert!(!p.is_fallback);
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
        assert!(
            !p.supports(VersionFeature::Dialogs),
            "1.21.4 predates dialogs (1.21.6)"
        );
    }

    #[test]
    fn dialogs_not_in_1_21_5() {
        let v = MinecraftVersion::parse("1.21.5").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(
            !p.supports(VersionFeature::Dialogs),
            "1.21.5 predates dialogs"
        );
    }

    #[test]
    fn dialogs_in_1_21_6() {
        let v = MinecraftVersion::parse("1.21.6").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(
            p.supports(VersionFeature::Dialogs),
            "1.21.6 introduced dialogs"
        );
    }

    #[test]
    fn dialogs_in_26_1() {
        let v = MinecraftVersion::parse("26.1").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(p.supports(VersionFeature::Dialogs), "26.1 supports dialogs");
    }

    #[test]
    fn dialogs_not_in_26x_unknown() {
        // Unknown 26.x minors (beyond the known table) use conservative caps.
        let v = MinecraftVersion::parse("26.99").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(
            !p.supports(VersionFeature::Dialogs),
            "26.99 is unverified — conservative profile must not claim dialog support"
        );
    }

    #[test]
    fn villager_trades_not_in_1_21_11() {
        // Villager trades are a 26.x-only registry, never backported to 1.21.x.
        let v = MinecraftVersion::parse("1.21.11").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(
            !p.supports(VersionFeature::VillagerTrades),
            "1.21.11 predates villager trades"
        );
    }

    #[test]
    fn villager_trades_in_26_1() {
        let v = MinecraftVersion::parse("26.1").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(
            p.supports(VersionFeature::VillagerTrades),
            "26.1 introduced villager trades"
        );
    }

    #[test]
    fn villager_trades_in_26_2() {
        let v = MinecraftVersion::parse("26.2").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(p.supports(VersionFeature::VillagerTrades));
    }

    #[test]
    fn villager_trades_not_in_26x_unknown() {
        let v = MinecraftVersion::parse("26.99").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(
            !p.supports(VersionFeature::VillagerTrades),
            "unverified 26.x profile must not claim villager trade support"
        );
    }

    #[test]
    fn function_macros_gated() {
        let old = MinecraftVersion::parse("1.20.1").unwrap();
        let p = VersionProfile::resolve(&old).unwrap();
        assert!(
            !p.supports(VersionFeature::FunctionMacros),
            "1.20.1 has no macros"
        );

        let new = MinecraftVersion::parse("1.20.2").unwrap();
        let p2 = VersionProfile::resolve(&new).unwrap();
        assert!(
            p2.supports(VersionFeature::FunctionMacros),
            "1.20.2 added macros"
        );
    }

    #[test]
    fn jukebox_songs_gated() {
        let old = MinecraftVersion::parse("1.20.6").unwrap();
        let p = VersionProfile::resolve(&old).unwrap();
        assert!(
            !p.supports(VersionFeature::JukeboxSongs),
            "1.20.x has no jukebox songs"
        );

        let new = MinecraftVersion::parse("1.21.0").unwrap();
        let p2 = VersionProfile::resolve(&new).unwrap();
        assert!(
            p2.supports(VersionFeature::JukeboxSongs),
            "1.21+ has jukebox songs"
        );
    }

    #[test]
    fn typed_capabilities_are_explicit() {
        let v = MinecraftVersion::parse("1.21.4").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(p.supports(VersionFeature::ItemComponents));
        assert!(p.supports(VersionFeature::FunctionMacros));
        assert!(!p.supports(VersionFeature::Dialogs));
    }

    #[test]
    fn capabilities_1_21_x() {
        let v = MinecraftVersion::parse("1.21.4").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(p.supports(VersionFeature::ItemComponents));
        assert!(p.supports(VersionFeature::DataComponents));
        assert!(p.supports(VersionFeature::FunctionMacros));
        assert!(p.supports(VersionFeature::Predicates));
        assert!(p.supports(VersionFeature::TrimAssets));
        assert!(p.supports(VersionFeature::JukeboxSongs));
        assert!(p.supports(VersionFeature::DamageTypes));
        assert!(p.supports(VersionFeature::ChatTypes));
        assert!(p.supports(VersionFeature::Enchantments));
    }

    #[test]
    fn capabilities_26x_fallback() {
        let v = MinecraftVersion::parse("26.99").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert!(p.is_fallback());
        assert!(
            !p.supports(VersionFeature::Dialogs),
            "conservative profile: dialogs=false"
        );
        assert!(p.supports(VersionFeature::CalendarSeries26));
    }

    fn assert_conservative_fallback_capabilities(p: &VersionProfile) {
        assert!(p.is_fallback());
        for feature in [
            VersionFeature::ItemComponents,
            VersionFeature::DataComponents,
            VersionFeature::Dialogs,
            VersionFeature::FunctionMacros,
            VersionFeature::Predicates,
            VersionFeature::ResourcePackOverlays,
            VersionFeature::TrimAssets,
            VersionFeature::JukeboxSongs,
            VersionFeature::DamageTypes,
            VersionFeature::ChatTypes,
            VersionFeature::Enchantments,
            VersionFeature::AnimalVariants,
            VersionFeature::VillagerTrades,
        ] {
            assert!(!p.supports(feature), "fallback must not claim {feature:?}");
        }
    }

    #[test]
    fn future_121_fallback_is_conservative() {
        let v = MinecraftVersion::parse("1.21.99").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert_eq!(p.data_pack_format(), 94);
        assert_eq!(p.resource_pack_format(), 75);
        assert_conservative_fallback_capabilities(&p);
    }

    #[test]
    fn future_26_fallback_is_conservative() {
        let v = MinecraftVersion::parse("26.99").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert_eq!(p.data_pack_format(), 107);
        assert_eq!(p.resource_pack_format(), 88);
        assert!(p.supports(VersionFeature::CalendarSeries26));
        assert_conservative_fallback_capabilities(&p);
    }

    #[test]
    fn future_26_patch_fallback_is_conservative() {
        for ver in ["26.1.99", "26.2.99"] {
            let v = MinecraftVersion::parse(ver).unwrap();
            let p = VersionProfile::resolve(&v).unwrap();
            assert_eq!(p.data_pack_format(), 107);
            assert_eq!(p.resource_pack_format(), 88);
            assert!(
                p.supports(VersionFeature::CalendarSeries26),
                "{ver} should still be recognized as a 26-series version"
            );
            assert_conservative_fallback_capabilities(&p);
        }
    }

    // ── resolve_strict ────────────────────────────────────────────────────────

    #[test]
    fn strict_known_version_ok() {
        let v = MinecraftVersion::parse("1.21.4").unwrap();
        assert!(VersionProfile::resolve_strict(&v).is_ok());
    }

    #[test]
    fn strict_known_26x_ok() {
        let v = MinecraftVersion::parse("26.1").unwrap();
        assert!(
            VersionProfile::resolve_strict(&v).is_ok(),
            "26.1 is a known version"
        );
        let v2 = MinecraftVersion::parse("26.2").unwrap();
        assert!(
            VersionProfile::resolve_strict(&v2).is_ok(),
            "26.2 is a known version"
        );
    }

    #[test]
    fn strict_unknown_26x_fails() {
        let v = MinecraftVersion::parse("26.99").unwrap();
        let err = VersionProfile::resolve_strict(&v).unwrap_err();
        assert!(
            matches!(err, VersionError::UnknownVersion { .. }),
            "expected UnknownVersion for 26.99, got {err:?}"
        );
    }

    #[test]
    fn strict_unknown_26_patch_fails() {
        for ver in ["26.1.99", "26.2.99"] {
            let v = MinecraftVersion::parse(ver).unwrap();
            let err = VersionProfile::resolve_strict(&v).unwrap_err();
            assert!(
                matches!(err, VersionError::UnknownVersion { .. }),
                "expected UnknownVersion for {ver}, got {err:?}"
            );
        }
    }

    #[test]
    fn strict_unknown_121x_fails() {
        let v = MinecraftVersion::parse("1.21.99").unwrap();
        let err = VersionProfile::resolve_strict(&v).unwrap_err();
        assert!(
            matches!(err, VersionError::UnknownVersion { .. }),
            "expected UnknownVersion for 1.21.99, got {err:?}"
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

    #[test]
    fn resource_pack_formats_1_21_series() {
        let cases = [
            ("1.21.0", 34u32),
            ("1.21.2", 42),
            ("1.21.4", 46),
            ("1.21.5", 55),
            ("1.21.6", 63),
            ("1.21.7", 64),
            ("1.21.9", 69),
            ("1.21.11", 75),
        ];
        for (ver, expected) in cases {
            let v = MinecraftVersion::parse(ver).unwrap();
            let p = VersionProfile::resolve(&v).unwrap();
            assert_eq!(
                p.resource_pack_format, expected,
                "wrong resource_pack_format for {ver}"
            );
        }
    }

    #[test]
    fn data_pack_formats_1_21_series() {
        let cases = [
            ("1.21.0", 48u32),
            ("1.21.2", 57),
            ("1.21.4", 61),
            ("1.21.5", 71),
            ("1.21.6", 80),
            ("1.21.7", 81),
            ("1.21.9", 88),
            ("1.21.11", 94),
        ];
        for (ver, expected) in cases {
            let v = MinecraftVersion::parse(ver).unwrap();
            let p = VersionProfile::resolve(&v).unwrap();
            assert_eq!(
                p.data_pack_format, expected,
                "wrong data_pack_format for {ver}"
            );
        }
    }

    #[test]
    fn pack_formats_26_series() {
        let cases = [
            ("26.1", 101u32, 84u32),
            ("26.1.2", 101, 84),
            ("26.2", 107, 88),
        ];
        for (ver, expected_data, expected_res) in cases {
            let v = MinecraftVersion::parse(ver).unwrap();
            let p = VersionProfile::resolve(&v).unwrap();
            assert_eq!(
                p.data_pack_format, expected_data,
                "wrong data_fmt for {ver}"
            );
            assert_eq!(
                p.resource_pack_format, expected_res,
                "wrong res_fmt for {ver}"
            );
            assert!(!p.is_fallback, "{ver} must be a known version");
        }
    }

    #[test]
    fn resource_pack_formats_1_18_series() {
        let v1 = MinecraftVersion::parse("1.18.1").unwrap();
        let p1 = VersionProfile::resolve(&v1).unwrap();
        assert_eq!(p1.resource_pack_format, 8);
        assert_eq!(p1.data_pack_format, 8);
        assert!(!p1.is_fallback);

        let v2 = MinecraftVersion::parse("1.18.2").unwrap();
        let p2 = VersionProfile::resolve(&v2).unwrap();
        assert_eq!(p2.resource_pack_format, 8);
        assert_eq!(p2.data_pack_format, 9);
        assert!(!p2.is_fallback);
    }

    #[test]
    fn conservative_fallback_uses_latest_res_fmt() {
        // Unknown versions use the latest known resource pack format (88, 26.2)
        // so generated packs are at least structurally valid.
        let v = MinecraftVersion::parse("1.22.0").unwrap();
        let p = VersionProfile::resolve(&v).unwrap();
        assert_eq!(p.resource_pack_format, 88);
        assert_eq!(p.data_pack_format, 107);
        assert!(p.is_fallback);
    }

    #[test]
    fn version_docs_track_latest_known_profile() {
        use std::{fs, path::Path};

        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace = manifest_dir
            .parent()
            .expect("sand-core should live under the workspace root");
        let docs = [
            workspace.join("book/src/reference/version-support.md"),
            workspace.join("sand-resourcepack/src/lib.rs"),
        ];
        let latest = VersionProfile::resolve(&MinecraftVersion::parse(LATEST_KNOWN).unwrap())
            .expect("LATEST_KNOWN must resolve");
        let latest_line = format!("latest known version is `{LATEST_KNOWN}`");
        let data_fmt = format!("data_fmt={}", latest.data_pack_format);
        let res_fmt = format!("res_fmt={}", latest.resource_pack_format);

        for path in docs {
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
            let lower_text = text.to_ascii_lowercase();
            assert!(
                lower_text.contains(&latest_line),
                "{} must mention {latest_line}",
                path.display()
            );
            assert!(
                text.contains(&data_fmt),
                "{} must mention {data_fmt}",
                path.display()
            );
            assert!(
                text.contains(&res_fmt),
                "{} must mention {res_fmt}",
                path.display()
            );
            assert!(
                lower_text.contains("conservative") && lower_text.contains("fallback"),
                "{} must explain conservative fallback behavior",
                path.display()
            );
        }
    }

    /// Regression for the default codegen contract (#118): the default
    /// `SAND_MC_VERSION` used by `sand-core/build.rs` must be a verified,
    /// codegen-available *known* profile (not a fallback), it must live in a
    /// single source of truth shared with `sand-version`, and it must stay
    /// distinct from the export/profile anchor `LATEST_KNOWN` so codegen and
    /// version-profile concerns are not conflated.
    #[test]
    fn default_codegen_version_contract() {
        // Single source of truth is `sand_version::DEFAULT_CODEGEN_VERSION`.
        assert_eq!(
            DEFAULT_CODEGEN_VERSION,
            sand_version::DEFAULT_CODEGEN_VERSION
        );
        assert!(!DEFAULT_CODEGEN_VERSION.is_empty());

        // The default target must resolve to a *known* (non-fallback) profile,
        // i.e. it is a verified version Sand can codegen against, not a guess.
        let v = MinecraftVersion::parse(DEFAULT_CODEGEN_VERSION)
            .expect("DEFAULT_CODEGEN_VERSION must parse");
        let profile = VersionProfile::resolve(&v)
            .expect("DEFAULT_CODEGEN_VERSION must resolve to a known profile");
        assert!(
            !profile.is_fallback,
            "DEFAULT_CODEGEN_VERSION ({DEFAULT_CODEGEN_VERSION}) must be a known, \
             verified codegen target, not a fallback profile"
        );

        // Codegen target ≠ export/profile anchor unless intentionally aligned.
        // They are allowed to differ; this assert documents the relationship
        // and fails loudly if someone conflates the two without intent.
        let _latest = LATEST_KNOWN;
    }
}
