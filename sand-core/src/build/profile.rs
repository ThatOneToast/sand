//! Build profiles: named configurations selected at `sand build`/`sand run`
//! time that a `sand.build.rs` script branches on.
//!
//! 🌍/🖥️ profiles themselves are neither world nor server content — they are
//! the selection mechanism a build script uses to decide *which* world and
//! server configuration to produce. See [`super::context::BuildContext`].

use sand_macros::api;
use std::fmt;

/// A named build profile.
///
/// Sand recognizes four well-known profiles out of the box —
/// [`BuildProfile::Dev`], [`BuildProfile::Test`], [`BuildProfile::Bench`],
/// [`BuildProfile::Release`] — plus arbitrary [`BuildProfile::Custom`] names
/// for project-specific needs (e.g. `"staging"`).
///
/// `sand build --profile <name>` / `sand run --profile <name>` select the
/// active profile; it defaults to `dev` for both commands. A `sand.build.rs`
/// script receives the resolved profile through [`super::context::BuildContext::profile`]
/// and branches on it:
///
/// ```
/// use sand_core::build::{BuildContext, BuildProfile};
///
/// fn describe(ctx: &BuildContext) -> &'static str {
///     if ctx.profile().is_dev() {
///         "flat, fast-iteration world"
///     } else if ctx.profile().is_release() {
///         "full vanilla noise world"
///     } else {
///         "other profile"
///     }
/// }
///
/// let ctx = BuildContext::new(BuildProfile::Dev);
/// assert_eq!(describe(&ctx), "flat, fast-iteration world");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::BuildProfile",
    module = "sand::build",
    summary = "BuildProfile selects which named world/server configuration a sand.build.rs script emits.",
    context = "sand build and sand run resolve one profile (default dev) and pass it to the project's build function via BuildContext.",
    minecraft = "The profile name has no direct Minecraft representation; it only steers which typed World/ServerConfig the build script constructs.",
    use_when = ["Branching worldgen or server settings on dev vs release vs a custom named profile"],
    avoid_when = ["Representing an actual Minecraft version or feature gate"],
    variants(Dev = "Fast local iteration profile: typically flat/void, auto-reset enabled.", Test = "Deterministic profile for automated testing: fixed seed, known structures.", Bench = "Profile for reproducible performance benchmarking: fixed seed and region.", Release = "The shipped configuration profile: typically full vanilla noise generation.", Custom = "Any other project-defined profile name, e.g. staging."),
    variant_fields(Custom = ["The custom profile's name."]),
    example = "if ctx.profile().is_dev() { /* flat world */ }"
)]
pub enum BuildProfile {
    /// Fast local iteration: typically a flat or void world, auto-reset
    /// enabled, minimal generation cost.
    Dev,
    /// Deterministic worlds for automated testing: typically flat with a
    /// fixed seed and known structures placed for test fixtures.
    Test,
    /// Worlds shaped for reproducible performance benchmarking: typically a
    /// fixed seed and a fixed, pre-generated region.
    Bench,
    /// The shipped configuration: typically full vanilla noise generation.
    Release,
    /// Any other project-defined profile name (e.g. `"staging"`).
    Custom(String),
}

impl BuildProfile {
    /// Parses a profile name from a CLI flag or environment variable value.
    /// Recognized names are case-insensitive; anything else becomes
    /// [`BuildProfile::Custom`].
    #[api(
        registry = sand_api_contract,
        path = "sand::build::BuildProfile::parse",
        module = "sand::build",
        summary = "Parses a profile name from a CLI flag or environment variable value.",
        context = "sand-cli resolves --profile/env var text into a BuildProfile before invoking a build script.",
        minecraft = "Recognized names map to the well-known profiles; anything else becomes BuildProfile::Custom.",
        use_when = ["Resolving a --profile flag or SAND_BUILD_PROFILE value"],
        avoid_when = ["Comparing two already-typed profiles"],
        params(name = "The raw profile name text."),
        returns = "The parsed BuildProfile.",
        example = "assert!(BuildProfile::parse(\"dev\").is_dev());"
    )]
    pub fn parse(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "dev" | "development" => BuildProfile::Dev,
            "test" => BuildProfile::Test,
            "bench" | "benchmark" => BuildProfile::Bench,
            "release" => BuildProfile::Release,
            _ => BuildProfile::Custom(name.to_string()),
        }
    }

    /// The canonical lowercase name for this profile (round-trips through
    /// [`BuildProfile::parse`]).
    #[api(
        registry = sand_api_contract,
        path = "sand::build::BuildProfile::name",
        module = "sand::build",
        summary = "Returns the canonical lowercase name for a profile.",
        context = "The name round-trips through BuildProfile::parse for diagnostics and CLI display.",
        minecraft = "Used only in Sand's own tooling output, never emitted into datapack content.",
        use_when = ["Printing or logging which profile is active"],
        avoid_when = ["Comparing profile identity; match on the enum instead"],
        returns = "The canonical lowercase profile name.",
        example = "assert_eq!(BuildProfile::Dev.name(), \"dev\");"
    )]
    pub fn name(&self) -> &str {
        match self {
            BuildProfile::Dev => "dev",
            BuildProfile::Test => "test",
            BuildProfile::Bench => "bench",
            BuildProfile::Release => "release",
            BuildProfile::Custom(name) => name,
        }
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::BuildProfile::is_dev",
        module = "sand::build",
        summary = "Reports whether this is the Dev profile.",
        context = "Build scripts branch worldgen choices on this predicate.",
        minecraft = "Selects fast-iteration world generation (e.g. flat) for local play.",
        use_when = ["Choosing fast-iteration worldgen in a build script"],
        avoid_when = ["Checking any other profile"],
        returns = "True for BuildProfile::Dev.",
        example = "assert!(BuildProfile::Dev.is_dev());"
    )]
    pub fn is_dev(&self) -> bool {
        matches!(self, BuildProfile::Dev)
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::BuildProfile::is_test",
        module = "sand::build",
        summary = "Reports whether this is the Test profile.",
        context = "Build scripts branch deterministic worldgen choices on this predicate.",
        minecraft = "Selects fixed-seed, fixture-bearing world generation for automated tests.",
        use_when = ["Choosing deterministic worldgen for automated tests"],
        avoid_when = ["Checking any other profile"],
        returns = "True for BuildProfile::Test.",
        example = "assert!(BuildProfile::Test.is_test());"
    )]
    pub fn is_test(&self) -> bool {
        matches!(self, BuildProfile::Test)
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::BuildProfile::is_bench",
        module = "sand::build",
        summary = "Reports whether this is the Bench profile.",
        context = "Build scripts branch reproducible-performance worldgen choices on this predicate.",
        minecraft = "Selects fixed-seed pre-generated regions for benchmarking.",
        use_when = ["Choosing benchmark worldgen"],
        avoid_when = ["Checking any other profile"],
        returns = "True for BuildProfile::Bench.",
        example = "assert!(BuildProfile::Bench.is_bench());"
    )]
    pub fn is_bench(&self) -> bool {
        matches!(self, BuildProfile::Bench)
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::BuildProfile::is_release",
        module = "sand::build",
        summary = "Reports whether this is the Release profile.",
        context = "Build scripts branch shipped-quality worldgen choices on this predicate.",
        minecraft = "Selects full vanilla noise generation for the shipped datapack.",
        use_when = ["Choosing shipped-quality worldgen"],
        avoid_when = ["Checking any other profile"],
        returns = "True for BuildProfile::Release.",
        example = "assert!(BuildProfile::Release.is_release());"
    )]
    pub fn is_release(&self) -> bool {
        matches!(self, BuildProfile::Release)
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::BuildProfile::is_custom",
        module = "sand::build",
        summary = "Reports whether this is a project-defined custom profile.",
        context = "Lets a build script fall back to a default branch for unrecognized profile names.",
        minecraft = "Has no Minecraft meaning by itself; gates custom build-script logic only.",
        use_when = ["Handling project-specific profile names like staging"],
        avoid_when = ["Checking one of the four well-known profiles"],
        returns = "True for BuildProfile::Custom(_).",
        example = "assert!(BuildProfile::parse(\"staging\").is_custom());"
    )]
    pub fn is_custom(&self) -> bool {
        matches!(self, BuildProfile::Custom(_))
    }
}

impl Default for BuildProfile {
    /// `sand build`/`sand run` default to `dev` when `--profile` is omitted.
    fn default() -> Self {
        BuildProfile::Dev
    }
}

impl fmt::Display for BuildProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_well_known_names_case_insensitively() {
        assert_eq!(BuildProfile::parse("DEV"), BuildProfile::Dev);
        assert_eq!(BuildProfile::parse("Release"), BuildProfile::Release);
        assert_eq!(BuildProfile::parse("bench"), BuildProfile::Bench);
        assert_eq!(BuildProfile::parse("test"), BuildProfile::Test);
    }

    #[test]
    fn unknown_names_become_custom() {
        assert_eq!(
            BuildProfile::parse("staging"),
            BuildProfile::Custom("staging".to_string())
        );
    }

    #[test]
    fn name_round_trips_through_parse() {
        for p in [
            BuildProfile::Dev,
            BuildProfile::Test,
            BuildProfile::Bench,
            BuildProfile::Release,
            BuildProfile::Custom("staging".into()),
        ] {
            assert_eq!(BuildProfile::parse(p.name()), p);
        }
    }

    #[test]
    fn default_is_dev() {
        assert_eq!(BuildProfile::default(), BuildProfile::Dev);
    }

    #[test]
    fn predicates_are_mutually_exclusive() {
        let p = BuildProfile::Release;
        assert!(p.is_release());
        assert!(!p.is_dev() && !p.is_test() && !p.is_bench() && !p.is_custom());
    }
}
