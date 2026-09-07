//! Minecraft version compatibility layer.
//!
//! Provides a single source of truth for version parsing, pack format lookup,
//! and feature flags across supported Minecraft Java 26.x+ versions.
//!
//! # Quick start
//! ```
//! use sand_core::version::{MinecraftVersion, VersionFeature, VersionProfile};
//!
//! let v = MinecraftVersion::parse("26.2").unwrap();
//! let profile = VersionProfile::resolve(&v).unwrap();
//! assert_eq!(profile.data_pack_format(), 107);
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
    example = "let version = MinecraftVersion::parse(\"26.2\")?;",
    variants(
        ParseError = "Carries text that cannot be parsed as a Minecraft Java version.",
        UnsupportedVersion = "Reports a release older than Sand's Minecraft 26.x baseline.",
        UnknownVersion = "Reports a parseable version that has no exact verified Sand profile."
    ),
    variant_fields(
        ParseError = ["The original malformed version text."],
        UnsupportedVersion(requested = "The parseable Minecraft release below version 26."),
        UnknownVersion(requested = "The parseable Minecraft version that lacks an exact verified profile.")
    )
)]
pub enum VersionError {
    /// The version string could not be parsed.
    #[error(
        "Invalid version '{0}': expected a Minecraft Java 26.x+ release such as '26.1', '26.2', '27.0', or 'latest'"
    )]
    ParseError(String),
    /// The version predates Sand's Minecraft Java 26.x baseline.
    #[error(
        "Unsupported Minecraft version '{requested}': Sand targets Minecraft Java 26.x and newer"
    )]
    UnsupportedVersion { requested: String },
    /// The version was parsed but is not in the known table.
    ///
    /// Use [`VersionProfile::resolve`] to inspect a conservative future-version
    /// fallback during local experimentation.
    #[error(
        "Unknown or unverified Minecraft version '{requested}'. Use VersionProfile::resolve to inspect a conservative fallback for local experimentation."
    )]
    UnknownVersion { requested: MinecraftVersion },
}

// ── MinecraftVersion ──────────────────────────────────────────────────────────

/// A parsed Minecraft Java Edition version.
///
/// Supports Minecraft's calendar-versioned 26.x and later series, plus the
/// special `latest` token which resolves to the newest known entry.
///
/// # Examples
/// ```
/// use sand_core::version::MinecraftVersion;
///
/// let target = MinecraftVersion::parse("26.1").unwrap();
/// let latest = MinecraftVersion::parse("latest").unwrap();
/// assert!(target.is_26_series());
/// assert!(latest.is_latest());
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
    example = "let target = MinecraftVersion::parse(\"26.2\")?;"
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
    /// Accepted formats include `"26"`, `"26.2"`, `"26.1.2"`,
    /// `"27.0"`, and `"latest"`.
    #[api(
        registry = sand_api_contract,
        path = "sand::version::MinecraftVersion::parse",
        aliases = ["sand::prelude::MinecraftVersion::parse"],
        module = "sand::version",
        summary = "Parses a Minecraft Java Edition release or the latest token.",
        context = "Parsing once at the configuration boundary lets later APIs reject malformed versions without carrying raw strings.",
        minecraft = "Accepts calendar 26.x and later releases plus latest for Sand's verified release table.",
        use_when = ["Reading a target version from configuration", "Creating a typed comparison minimum"],
        avoid_when = ["Reusing a VersionProfile that is already resolved"],
        params(s = "The release text, such as 26.2, 27.0, or latest."),
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
        if let VersionKind::Specific { major, .. } = kind
            && major < 26
        {
            return Err(VersionError::UnsupportedVersion {
                requested: s.to_string(),
            });
        }
        Ok(Self { kind })
    }

    /// Returns `true` if this is the `latest` token.
    #[api(registry = sand_api_contract, path = "sand::version::MinecraftVersion::is_latest", aliases = ["sand::prelude::MinecraftVersion::is_latest"], module = "sand::version", summary = "Checks whether this value is Sand's latest token.", context = "The latest token resolves through the current verified release table rather than storing a concrete release in author configuration.", minecraft = "Latest selects Sand's newest verified Minecraft target during profile resolution.", use_when = ["Preserving a caller's latest-versus-explicit choice"], avoid_when = ["Determining whether a concrete release supports a feature"], returns = "True when this value was parsed from latest.", example = "assert!(MinecraftVersion::parse(\"latest\")?.is_latest());")]
    pub fn is_latest(&self) -> bool {
        matches!(self.kind, VersionKind::Latest)
    }

    /// Returns `true` for the new `26.x` calendar series.
    #[api(registry = sand_api_contract, path = "sand::version::MinecraftVersion::is_26_series", aliases = ["sand::prelude::MinecraftVersion::is_26_series"], module = "sand::version", summary = "Checks whether this is a calendar-series 26.x Minecraft release.", context = "This distinguishes Sand's current verified family from representable later calendar releases.", minecraft = "Calendar releases use names such as 26.1 and 26.2.", use_when = ["Selecting behavior intentionally specific to Mojang's 26.x release series"], avoid_when = ["Checking a feature represented by VersionFeature"], returns = "True for an explicit 26.x release.", example = "assert!(MinecraftVersion::parse(\"26.2\")?.is_26_series());")]
    pub fn is_26_series(&self) -> bool {
        matches!(self.kind, VersionKind::Specific { major: 26, .. })
    }

    /// Return major, minor, patch components if this is a specific version.
    #[api(registry = sand_api_contract, path = "sand::version::MinecraftVersion::components", aliases = ["sand::prelude::MinecraftVersion::components"], module = "sand::version", summary = "Returns numeric components for an explicit release.", context = "The latest token is intentionally not a fixed numeric version until profile resolution chooses Sand's current verified anchor.", minecraft = "Minecraft releases are compared as major, minor, and patch numbers when they are explicit.", use_when = ["Displaying or adapting an explicit release number"], avoid_when = ["Comparing targets; use is_at_least instead"], returns = "The major, minor, and patch components, or None for latest.", example = "assert_eq!(MinecraftVersion::parse(\"26.2\")?.components(), Some((26, 2, 0)));" )]
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
    /// `latest` resolves to Sand's newest known version before comparison.
    ///
    /// # Examples
    /// ```
    /// use sand_core::version::MinecraftVersion;
    ///
    /// let v = MinecraftVersion::parse("26.2").unwrap();
    /// assert!(v.is_at_least(&MinecraftVersion::parse("26.1").unwrap()));
    /// assert!(!v.is_at_least(&MinecraftVersion::parse("27.0").unwrap()));
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
        example = "assert!(MinecraftVersion::parse(\"26.2\")?.is_at_least(&MinecraftVersion::parse(\"26.1\")?));"
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
/// requested version. Unknown future versions receive conservative
/// capabilities and must not be treated as schema-verified.
///
/// # Examples
/// ```
/// use sand_core::version::{MinecraftVersion, VersionFeature, VersionProfile};
///
/// // Known 26.x version → exact profile
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
    minecraft = "Maps a Minecraft Java release to its datapack format and availability of versioned vanilla features.",
    use_when = ["Resolving a target release before authoring version-aware content", "Checking a VersionFeature before emitting optional content"],
    avoid_when = ["Constructing contradictory feature flags or pack formats by hand"],
    example = "let profile = VersionProfile::resolve(&MinecraftVersion::parse(\"26.2\")?)?;"
)]
pub struct VersionProfile {
    /// The version that was requested.
    requested: MinecraftVersion,
    /// Human-readable resolved name.
    resolved_name: String,
    /// Data pack format number for `pack.mcmeta`.
    data_pack_format: u32,
    /// Whether this version supports item components.
    supports_item_components: bool,
    /// Whether this version supports `data modify` components.
    supports_data_components: bool,
    /// Whether this is the new 26.x calendar-versioned series.
    supports_26_series: bool,
    /// Whether this version supports data-driven dialogs.
    supports_dialogs: bool,
    /// Whether this version supports function macros.
    supports_function_macros: bool,
    /// Whether this version supports predicates.
    supports_predicates: bool,
    /// Whether this version supports trim assets.
    supports_trim_assets: bool,
    /// Whether this version supports jukebox song components.
    supports_jukebox_songs: bool,
    /// Whether this version supports damage type registries.
    supports_damage_types: bool,
    /// Whether this version supports chat type registries.
    supports_chat_types: bool,
    /// Whether this version supports enchantment data components.
    supports_enchantments: bool,
    /// Whether this version supports biome-scoped animal variant registries —
    /// `chicken_variant`, `cow_variant`, and `pig_variant`.
    supports_animal_variants: bool,
    /// Whether this version supports the data-driven Villager/Wandering
    /// Trader trade registries — `villager_trade` and `trade_set`.
    supports_villager_trades: bool,
    /// When `true` the profile was resolved via a conservative fallback because
    /// the exact version was not in the known table. Users should verify and
    /// must not assume its schema is verified.
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
/// Obtain via [`VersionProfile::datapack_metadata`].
///
/// # Example
/// ```
/// use sand_core::version::{MinecraftVersion, VersionProfile};
///
/// let v = MinecraftVersion::parse("26.2").unwrap();
/// let p = VersionProfile::resolve(&v).unwrap();
/// let meta = p.datapack_metadata();
/// assert_eq!(meta.pack_format(), 107);
/// assert!(!meta.is_fallback());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[api(
    registry = sand_api_contract,
    path = "sand::version::PackMetadata",
    module = "sand::version",
    summary = "Identifies the pack format selected for one Minecraft pack root.",
    context = "Metadata is obtained from a resolved VersionProfile rather than constructed by hand.",
    minecraft = "Supplies the pack_format value written to a pack.mcmeta file.",
    use_when = ["Writing datapack metadata for a resolved target"],
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
    #[api(registry = sand_api_contract, path = "sand::version::PackMetadata::pack_format", module = "sand::version", summary = "Returns the selected pack.mcmeta format number.", context = "The format is kept with its fallback status so export code cannot accidentally separate them.", minecraft = "Writes the integer Minecraft reads from pack.pack_format.", use_when = ["Serializing the pack section of pack.mcmeta"], avoid_when = ["Selecting a format without resolving a VersionProfile"], returns = "The exact pack format number for this pack root.", example = "assert_eq!(profile.datapack_metadata().pack_format(), 107);")]
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
        ItemComponents = "The structured item-component system.",
        DataComponents = "The data command support for structured item components.",
        CalendarSeries26 = "Mojang's 26.x calendar release series.",
        Dialogs = "Data-driven Minecraft dialogs.",
        FunctionMacros = "Function macro substitution using $() syntax.",
        Predicates = "Reusable predicate JSON resources supported by Sand's target range.",
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
            supports_item_components: caps.item_components,
            supports_data_components: caps.data_components,
            supports_26_series: supports_26,
            supports_dialogs: caps.dialogs,
            supports_function_macros: caps.function_macros,
            supports_predicates: caps.predicates,
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
    #[api(registry = sand_api_contract, path = "sand::version::VersionProfile::data_pack_format", aliases = ["sand::prelude::VersionProfile::data_pack_format"], module = "sand::version", summary = "Returns the datapack pack.mcmeta format for this target.", context = "The profile owns format selection so datapack metadata cannot drift from capability checks.", minecraft = "Supplies the pack_format accepted for data packs by the target Minecraft release.", use_when = ["Inspecting datapack metadata before export"], avoid_when = ["Guessing a format from an unvalidated version string"], returns = "The target datapack format number.", example = "let format = profile.data_pack_format();")]
    pub fn data_pack_format(&self) -> u32 {
        self.data_pack_format
    }

    /// Whether Sand used a conservative profile because the exact release is unknown.
    #[api(registry = sand_api_contract, path = "sand::version::VersionProfile::is_fallback", aliases = ["sand::prelude::VersionProfile::is_fallback"], module = "sand::version", summary = "Reports whether this profile is conservative rather than exact.", context = "Sand never guesses new Minecraft capabilities: unverified releases use a fallback profile that callers can surface or reject.", minecraft = "Fallback targets do not claim availability for version-sensitive vanilla content.", use_when = ["Warning about or rejecting an unverified target"], avoid_when = ["Treating a syntactically valid future release as fully supported"], returns = "True when the requested release has no exact verified profile.", example = "if profile.is_fallback() { /* require an explicit override */ }")]
    pub fn is_fallback(&self) -> bool {
        self.is_fallback
    }

    /// Returns whether this target supports one typed Minecraft capability.
    #[api(registry = sand_api_contract, path = "sand::version::VersionProfile::supports", aliases = ["sand::prelude::VersionProfile::supports"], module = "sand::version", summary = "Checks one typed Minecraft capability for this target.", context = "A single typed query keeps feature gating complete and avoids raw string keys that silently misspell to false.", minecraft = "Reports whether the exact target schema contains the requested vanilla feature family.", use_when = ["Validating schema-dependent authored content"], avoid_when = ["Comparing an arbitrary release threshold; use MinecraftVersion::is_at_least"], params(feature = "The explicit Minecraft capability to check."), returns = "True when the resolved target supports that capability.", example = "assert!(profile.supports(VersionFeature::ItemComponents));")]
    pub fn supports(&self, feature: VersionFeature) -> bool {
        match feature {
            VersionFeature::ItemComponents => self.supports_item_components,
            VersionFeature::DataComponents => self.supports_data_components,
            VersionFeature::CalendarSeries26 => self.supports_26_series,
            VersionFeature::Dialogs => self.supports_dialogs,
            VersionFeature::FunctionMacros => self.supports_function_macros,
            VersionFeature::Predicates => self.supports_predicates,
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
    /// explicitly listed in the known-version table, including future calendar
    /// versions not yet verified by Sand.
    ///
    /// # Examples
    /// ```
    /// use sand_core::version::{MinecraftVersion, VersionProfile};
    ///
    /// // Known version → OK
    /// let v = MinecraftVersion::parse("26.2").unwrap();
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
    #[api(registry = sand_api_contract, path = "sand::version::VersionProfile::datapack_metadata", aliases = ["sand::prelude::VersionProfile::datapack_metadata"], module = "sand::version", summary = "Builds metadata for a target datapack root.", context = "The metadata carries both the resolved format and its fallback status as one immutable value.", minecraft = "Produces the pack_format Minecraft reads from a datapack's pack.mcmeta.", use_when = ["Writing datapack metadata during export"], avoid_when = ["Selecting a format without resolving the target profile"], returns = "The selected datapack PackMetadata.", example = "let metadata = profile.datapack_metadata();")]
    pub fn datapack_metadata(&self) -> PackMetadata {
        PackMetadata {
            pack_format: self.data_pack_format,
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
    item_components: bool,
    data_components: bool,
    dialogs: bool,
    function_macros: bool,
    predicates: bool,
    trim_assets: bool,
    jukebox_songs: bool,
    damage_types: bool,
    chat_types: bool,
    enchantments: bool,
    animal_variants: bool,
    villager_trades: bool,
    is_fallback: bool,
}

impl VersionCaps {
    fn known(data_fmt: u32) -> Self {
        Self {
            data_fmt,
            item_components: true,
            data_components: true,
            dialogs: true,
            function_macros: true,
            predicates: true,
            trim_assets: true,
            jukebox_songs: true,
            damage_types: true,
            chat_types: true,
            enchantments: true,
            animal_variants: true,
            villager_trades: true,
            is_fallback: false,
        }
    }

    /// Conservative profile for a 26.x+ version not explicitly verified.
    fn conservative() -> Self {
        Self {
            data_fmt: 107,
            item_components: false,
            data_components: false,
            dialogs: false,
            function_macros: false,
            predicates: false,
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

/// Look up capabilities for the supported 26.x+ era.
///
/// Exact schema claims exist only for versions Sand has verified. Parseable
/// future calendar versions remain representable but resolve conservatively.
fn lookup(major: u32, minor: u32, patch: u32) -> VersionCaps {
    match (major, minor, patch) {
        (26, 2, 0) => VersionCaps::known(107),
        (26, 1, 0..=2) => VersionCaps::known(101),
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
    /// The resolved version string (for example `"26.2"`).
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

    #[test]
    fn parses_supported_calendar_versions_and_latest() {
        let cases = [
            ("26", Some((26, 0, 0))),
            ("26.1", Some((26, 1, 0))),
            ("26.1.2", Some((26, 1, 2))),
            ("27.0", Some((27, 0, 0))),
            ("latest", None),
        ];
        for (input, expected) in cases {
            let version = MinecraftVersion::parse(input).unwrap();
            assert_eq!(version.components(), expected, "{input}");
        }
    }

    #[test]
    fn rejects_pre_26_and_malformed_versions() {
        for input in ["0", "10.4", "25.9"] {
            assert!(
                matches!(
                    MinecraftVersion::parse(input),
                    Err(VersionError::UnsupportedVersion { .. })
                ),
                "{input}"
            );
        }
        for input in ["abc", "26.x", "26.1.2.3"] {
            assert!(
                matches!(
                    MinecraftVersion::parse(input),
                    Err(VersionError::ParseError(_))
                ),
                "{input}"
            );
        }
    }

    #[test]
    fn known_profiles_and_future_fallback_are_table_driven() {
        for (version, format) in [
            ("26.1", 101),
            ("26.1.1", 101),
            ("26.1.2", 101),
            ("26.2", 107),
        ] {
            let profile =
                VersionProfile::resolve_strict(&MinecraftVersion::parse(version).unwrap()).unwrap();
            assert_eq!(profile.data_pack_format(), format, "{version}");
            assert!(!profile.is_fallback(), "{version}");
            for feature in [
                VersionFeature::ItemComponents,
                VersionFeature::DataComponents,
                VersionFeature::CalendarSeries26,
                VersionFeature::Dialogs,
                VersionFeature::FunctionMacros,
                VersionFeature::Predicates,
                VersionFeature::TrimAssets,
                VersionFeature::JukeboxSongs,
                VersionFeature::DamageTypes,
                VersionFeature::ChatTypes,
                VersionFeature::Enchantments,
                VersionFeature::AnimalVariants,
                VersionFeature::VillagerTrades,
            ] {
                assert!(profile.supports(feature), "{version} lacks {feature:?}");
            }
        }

        for version in ["26.3", "26.2.1", "27.0"] {
            let parsed = MinecraftVersion::parse(version).unwrap();
            let profile = VersionProfile::resolve(&parsed).unwrap();
            assert!(profile.is_fallback(), "{version}");
            assert_eq!(profile.data_pack_format(), 107, "{version}");
            assert!(
                VersionProfile::resolve_strict(&parsed).is_err(),
                "{version}"
            );
            assert!(!profile.supports(VersionFeature::Dialogs), "{version}");
        }
    }

    #[test]
    fn latest_resolves_to_verified_anchor() {
        let profile =
            VersionProfile::resolve_strict(&MinecraftVersion::parse("latest").unwrap()).unwrap();
        assert_eq!(profile.data_pack_format(), 107);
        assert!(profile.resolved_name().contains(LATEST_KNOWN));
        assert_eq!(DEFAULT_CODEGEN_VERSION, LATEST_KNOWN);
    }

    #[test]
    fn export_resolution_rejects_pre_26_and_preserves_future_fallbacks() {
        assert!(resolve_export_caps("25.9").is_err());

        let future = resolve_export_caps("27.0").unwrap();
        assert!(future.is_fallback);
        assert!(
            !future
                .caps
                .supports(sand_version::ComponentFeature::Dialogs)
        );

        let current = resolve_export_caps("26.2").unwrap();
        assert!(!current.is_fallback);
        assert_eq!(current.version, "26.2.0");
    }
}
