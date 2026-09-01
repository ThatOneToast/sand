//! Build-time context handed to a `sand.build.rs` script.

use super::profile::BuildProfile;
use sand_macros::api;

/// Context passed to a project's `sand.build.rs` entry point.
///
/// Carries the resolved [`BuildProfile`] (selected via `sand build --profile`
/// / `sand run --profile`, defaulting to `dev`) plus the target Minecraft
/// version string being built against, so a build script can branch on both.
///
/// ```
/// use sand_core::build::{BuildContext, BuildProfile};
///
/// let ctx = BuildContext::new(BuildProfile::Release).with_mc_version("26.2");
/// assert!(ctx.profile().is_release());
/// assert_eq!(ctx.mc_version(), "26.2");
/// ```
#[derive(Debug, Clone)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::BuildContext",
    module = "sand::build",
    summary = "BuildContext carries the resolved build profile and target Minecraft version into a sand.build.rs script.",
    context = "sand-cli constructs one BuildContext per invocation and passes it to the project's build function.",
    minecraft = "Neither field is written into the datapack directly; they steer which World/ServerConfig the script builds.",
    use_when = ["Reading the active profile or target version inside a build script"],
    avoid_when = ["Storing arbitrary project state; construct a fresh SandBuild instead"],
    example = "let ctx = BuildContext::new(BuildProfile::Dev);"
)]
pub struct BuildContext {
    profile: BuildProfile,
    mc_version: String,
}

impl BuildContext {
    /// Creates a context for the given profile. Defaults `mc_version` to
    /// `"latest"`; `sand-cli` overrides it with the resolved version from
    /// `sand.toml` before invoking the build script.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::BuildContext::new",
        module = "sand::build",
        summary = "Creates a build context for the given profile.",
        context = "sand-cli calls this once per sand build/sand run invocation before running the project's build function.",
        minecraft = "Defaults mc_version to \"latest\"; sand-cli overrides it with the resolved version from sand.toml.",
        use_when = ["Constructing the context sand-cli passes to a build script"],
        avoid_when = ["Constructing a context inside the build script itself"],
        params(profile = "The resolved build profile."),
        returns = "A new BuildContext with mc_version defaulted to latest.",
        example = "let ctx = BuildContext::new(BuildProfile::Dev);"
    )]
    pub fn new(profile: BuildProfile) -> Self {
        Self {
            profile,
            mc_version: "latest".to_string(),
        }
    }

    /// Sets the target Minecraft version string (builder-style).
    #[api(
        registry = sand_api_contract,
        path = "sand::build::BuildContext::with_mc_version",
        module = "sand::build",
        summary = "Overrides the target Minecraft version string.",
        context = "sand-cli sets this from sand.toml's resolved mc_version before invoking the build function.",
        minecraft = "The version string gates which VersionProfile validation applies to the produced world resources.",
        use_when = ["Targeting a specific Minecraft version from sand-cli"],
        avoid_when = ["Guessing the version inside a build script"],
        params(mc_version = "The target Minecraft version string, e.g. \"26.2\"."),
        returns = "This context with mc_version updated.",
        example = "let ctx = BuildContext::new(BuildProfile::Dev).with_mc_version(\"26.2\");"
    )]
    pub fn with_mc_version(mut self, mc_version: impl Into<String>) -> Self {
        self.mc_version = mc_version.into();
        self
    }

    /// The active build profile.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::BuildContext::profile",
        module = "sand::build",
        summary = "Returns the active build profile.",
        context = "Build scripts branch worldgen and server settings on this value.",
        minecraft = "Has no direct Minecraft effect; only steers which typed builders the script calls.",
        use_when = ["Branching a build script on dev/test/bench/release"],
        avoid_when = ["Reading the Minecraft version"],
        returns = "A reference to the active BuildProfile.",
        example = "assert!(BuildContext::new(BuildProfile::Dev).profile().is_dev());"
    )]
    pub fn profile(&self) -> &BuildProfile {
        &self.profile
    }

    /// The target Minecraft version string (e.g. `"26.2"`).
    #[api(
        registry = sand_api_contract,
        path = "sand::build::BuildContext::mc_version",
        module = "sand::build",
        summary = "Returns the target Minecraft version string.",
        context = "Build-time validation checks generated world resources against this version's VersionProfile.",
        minecraft = "Matches the mc_version sand.toml resolves (or an explicit override), not a datapack field itself.",
        use_when = ["Reading which Minecraft version a build script is targeting"],
        avoid_when = ["Reading the active build profile"],
        returns = "The target Minecraft version string.",
        example = "assert_eq!(BuildContext::new(BuildProfile::Dev).with_mc_version(\"26.2\").mc_version(), \"26.2\");"
    )]
    pub fn mc_version(&self) -> &str {
        &self.mc_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_mc_version_to_latest() {
        let ctx = BuildContext::new(BuildProfile::Dev);
        assert_eq!(ctx.mc_version(), "latest");
    }

    #[test]
    fn with_mc_version_overrides_it() {
        let ctx = BuildContext::new(BuildProfile::Dev).with_mc_version("1.21.4");
        assert_eq!(ctx.mc_version(), "1.21.4");
    }
}
