//! [`SandBuild`] — the top-level value a `sand.build.rs` script returns.

use super::server::ServerConfig;
use super::validate::{BuildDiagnostic, validate};
use super::world::World;
use sand_macros::api;

/// The complete typed build-time configuration for one profile: a
/// [`World`] (🌍 lowers to datapack resources) and an optional
/// [`ServerConfig`] (🖥️ consumed only by `sand run`).
///
/// A `sand.build.rs` script's entry point returns a `SandBuild`:
///
/// ```
/// use sand_core::build::{
///     BuildContext, Dimension, DimensionSlot, DimensionType, FlatGenerator, FlatLayer,
///     Generator, SandBuild, Spawn, World,
/// };
/// use sand_components::resource_location::ResourceLocation;
///
/// fn build(ctx: &BuildContext) -> SandBuild {
///     let overworld = if ctx.profile().is_dev() {
///         Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
///             Generator::Flat(FlatGenerator::new(vec![FlatLayer::new(
///                 ResourceLocation::new("minecraft", "grass_block").unwrap(),
///                 1,
///             )])),
///         )
///     } else {
///         Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld)
///     };
///
///     SandBuild::new().world(
///         World::new()
///             .spawn(Spawn::at(0, 65, 0))
///             .dimensions(sand_core::build::Dimensions::new().with(overworld)),
///     )
/// }
///
/// let ctx = BuildContext::new(sand_core::build::BuildProfile::Dev);
/// let built = build(&ctx);
/// assert!(built.validate().is_ok());
/// ```
#[derive(Debug, Clone, Default)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::SandBuild",
    module = "sand::build",
    summary = "SandBuild is the top-level value a sand.build.rs script's build function returns.",
    context = "Combines an optional World (datapack) and an optional ServerConfig (local dev server only) for one build profile.",
    minecraft = "World lowers into the exported datapack; ServerConfig is consumed only by sand run and never written into it.",
    use_when = ["Returning a project's typed world/server configuration from sand.build.rs"],
    avoid_when = ["Configuring a single dimension directly; build a World first"],
    example = "SandBuild::new().world(World::new());"
)]
pub struct SandBuild {
    pub(crate) world: Option<World>,
    pub(crate) server: Option<ServerConfig>,
}

impl SandBuild {
    #[api(
        registry = sand_api_contract,
        path = "sand::build::SandBuild::new",
        module = "sand::build",
        summary = "Creates an empty SandBuild with no World or ServerConfig configured.",
        context = "Starting point for a build script before chaining .world(...) and/or .server(...).",
        minecraft = "Produces no datapack resources and no server-config output until a World/ServerConfig is set.",
        use_when = ["Starting a build script's return value"],
        avoid_when = ["Configuring an already-built SandBuild's World; use .world(...)"],
        returns = "An empty SandBuild.",
        example = "SandBuild::new();"
    )]
    pub fn new() -> Self {
        Self::default()
    }

    /// 🌍 World (datapack).
    #[api(
        registry = sand_api_contract,
        path = "sand::build::SandBuild::world",
        module = "sand::build",
        summary = "Sets this build's World (datapack) configuration.",
        context = "The World's dimensions, spawn, border, gamerules, time, and weather all lower into the exported datapack.",
        minecraft = "Populates the dimension/function/tag resources SandBuild::validate and lower_world operate on.",
        use_when = ["Configuring dimensions, spawn, border, gamerules, time, or weather"],
        avoid_when = ["Configuring server-only settings; use .server(...)"],
        params(world = "The world configuration."),
        returns = "This build with the World set.",
        example = "SandBuild::new().world(World::new());"
    )]
    pub fn world(mut self, world: World) -> Self {
        self.world = Some(world);
        self
    }

    /// 🖥️ Server (host) — see [`ServerConfig`]'s docs.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::SandBuild::server",
        module = "sand::build",
        summary = "Sets this build's ServerConfig (local dev server only).",
        context = "Consumed only by sand run; never written into the exported datapack.",
        minecraft = "Has no datapack representation at all — see ServerConfig's own docs for why.",
        use_when = ["Configuring sand run's view distance, simulation distance, difficulty default, online-mode, or world-reset policy"],
        avoid_when = ["Configuring anything that should travel with the datapack; use .world(...)"],
        params(server = "The server integration configuration."),
        returns = "This build with the ServerConfig set.",
        example = "SandBuild::new().server(ServerConfig::new());"
    )]
    pub fn server(mut self, server: ServerConfig) -> Self {
        self.server = Some(server);
        self
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::SandBuild::world_ref",
        module = "sand::build",
        summary = "Returns this build's configured World, if any.",
        context = "Used by lowering and sand-cli to check whether a World was configured before writing datapack resources.",
        minecraft = "Reflects only what .world(...) set; never populated implicitly.",
        use_when = ["Inspecting whether a SandBuild configured a World"],
        avoid_when = ["Mutating the build; use .world(...) to replace it"],
        returns = "The configured World, or None.",
        example = "assert!(SandBuild::new().world_ref().is_none());"
    )]
    pub fn world_ref(&self) -> Option<&World> {
        self.world.as_ref()
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::SandBuild::server_ref",
        module = "sand::build",
        summary = "Returns this build's configured ServerConfig, if any.",
        context = "Used by sand run to apply local dev-server settings after a successful build.",
        minecraft = "Reflects only what .server(...) set; has no datapack effect either way.",
        use_when = ["Inspecting whether a SandBuild configured a ServerConfig"],
        avoid_when = ["Mutating the build; use .server(...) to replace it"],
        returns = "The configured ServerConfig, or None.",
        example = "assert!(SandBuild::new().server_ref().is_none());"
    )]
    pub fn server_ref(&self) -> Option<&ServerConfig> {
        self.server.as_ref()
    }

    /// Validates this build. See [`validate`] for the checks performed.
    #[api(
        registry = sand_api_contract,
        path = "sand::build::SandBuild::validate",
        module = "sand::build",
        summary = "Validates this build's World against structural and range checks.",
        context = "Called by run_and_print before lowering; a build script can also call it directly in its own tests.",
        minecraft = "Rejects, with a pointed diagnostic, world resources that would fail to load or exceed vanilla limits (e.g. an oversized world border).",
        use_when = ["Checking a SandBuild before lowering or in a project's own tests"],
        avoid_when = ["Lowering an already-validated build; use lower_world"],
        returns = "Ok(()) if every check passes, or every collected BuildDiagnostic.",
        example = "assert!(SandBuild::new().validate().is_ok());"
    )]
    pub fn validate(&self) -> Result<(), Vec<BuildDiagnostic>> {
        validate(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_build_validates() {
        assert!(SandBuild::new().validate().is_ok());
    }

    #[test]
    fn server_and_world_are_independently_settable() {
        let built = SandBuild::new()
            .world(World::new())
            .server(ServerConfig::new().view_distance(6));
        assert!(built.world_ref().is_some());
        assert_eq!(built.server_ref().unwrap().get_view_distance(), 6);
    }
}
