//! Typed, build-time world and server configuration (issue #317).
//!
//! This is the implementation behind `sand::build`. Authors should not
//! import `sand_core` directly — see the crate-level docs and
//! `docs/architecture/adr-001-crate-boundaries.md`.
//!
//! # World vs. Server
//!
//! Sand's shipped artifact is a **datapack**. Anything that ends up inside
//! it (dimension definitions, world generation, gamerules/time/weather set
//! via pack functions) is a 🌍 **World (datapack)** concern — it travels
//! with the datapack and works identically in singleplayer, LAN, realms, or
//! any vanilla-compatible server.
//!
//! A few settings ([`ServerConfig`]'s fields) have **no datapack
//! representation at all** — view distance, simulation distance, a
//! difficulty *default*, online-mode, and world-reset policy are
//! `server.properties`-equivalent concepts or `sand run`'s own local
//! bootstrap behavior. These are 🖥️ **Server (host)**-only: they configure
//! Sand's local dev server (or document a value you must set by hand on a
//! real dedicated server) and are never written into the exported datapack.
//! See [`ServerConfig`]'s module docs for the full explanation.
//!
//! # Entry point
//!
//! A `sand.build.rs` script exposes a function
//! `fn build(ctx: &BuildContext) -> SandBuild`, compiled by `sand-cli` as
//! the `sand_build_world` binary (see `sand add worldbuild`) and invoked
//! during `sand build`/`sand run`. See the "Build scripts" mdBook chapter.

use sand_macros::api;
mod context;
mod dimension;
mod generator;
mod profile;
mod registry;
mod resources;
mod sand_build;
mod server;
mod validate;
mod world;

pub use context::BuildContext;
pub use dimension::{Dimension, DimensionSlot, DimensionType, Dimensions};
pub use generator::{
    BiomeSource, FlatGenerator, FlatLayer, Generator, NoiseGenerator, NoiseSettingsRef,
    VanillaNoiseSettings,
};
pub use profile::BuildProfile;
pub use resources::{WorldResource, lower as lower_world};
pub use sand_build::SandBuild;
pub use server::{Difficulty, ServerConfig, WorldResetPolicy};
pub use validate::BuildDiagnostic;
pub use world::{
    Seed, Spawn, SpawnPlatform, TimeConfig, WeatherConfig, World, WorldBorder, WorldPreset,
};

/// Runs a project's `build` function against the given profile, validates
/// the result, and prints the lowered [`WorldResource`]s plus, if present,
/// the [`ServerConfig`] as one JSON object on stdout.
///
/// This is what the generated `sand_build_world` binary's `main` calls
/// (mirroring `try_export_components_json`'s role for the ordinary
/// `sand_export` binary). Exits the process with a non-zero status and a
/// diagnostic report on stderr if validation fails.
#[api(
    registry = sand_api_contract,
    path = "sand::build::run_and_print",
    module = "sand::build",
    summary = "Runs a project's build function against the given profile, validates the result, and prints its lowered resources as JSON.",
    context = "Called by the generated sand_build_world binary's main; mirrors try_export_components_json's role for the ordinary sand_export binary.",
    minecraft = "On success prints one JSON object (world resources plus an optional server_config) that sand-cli parses and writes; on validation failure exits non-zero with diagnostics on stderr.",
    use_when = ["Implementing the generated sand_build_world binary's entry point"],
    avoid_when = ["Calling this directly from ordinary datapack authoring code"],
    params(namespace = "The project's pack namespace.", ctx = "The resolved build context.", build_fn = "The project's sand.build.rs entry point."),
    returns = "Never returns; exits the process.",
    example = "run_and_print(\"my_pack\", BuildContext::new(BuildProfile::Dev), build);"
)]
pub fn run_and_print<F>(namespace: &str, ctx: BuildContext, build_fn: F) -> !
where
    F: FnOnce(&BuildContext) -> SandBuild,
{
    let built = build_fn(&ctx);
    if let Err(diagnostics) = built.validate_for_context(&ctx) {
        eprintln!("sand.build.rs produced an invalid world configuration:");
        for d in &diagnostics {
            eprintln!("  - {d}");
        }
        std::process::exit(1);
    }

    let resources = resources::lower(namespace, &built);
    let server = built.server_ref().cloned();
    // 🖥️ Server (host) only, despite living on World — see `Seed`'s docs.
    // `sand run` applies a fixed seed as server.properties' `level-seed`
    // when creating a fresh local world; a datapack has no way to set an
    // existing world's seed.
    let seed = built
        .world_ref()
        .and_then(|w| w.seed_ref())
        .and_then(|s| match s {
            world::Seed::Fixed(value) => Some(*value),
            world::Seed::Random => None,
        });
    // 🖥️ Server (host) only, despite living on World — see `WorldPreset`'s
    // docs. `sand run` applies this as server.properties' `level-type` when
    // creating a fresh local world; a datapack has no way to apply a preset
    // to an existing world.
    let level_type = built.world_ref().map(|w| w.preset_ref().level_type());
    let output = serde_json::json!({
        "resources": resources,
        "server_config": server.map(|s| serde_json::json!({
            "view_distance": s.get_view_distance(),
            "simulation_distance": s.get_simulation_distance(),
            "difficulty": s.get_difficulty().as_str(),
            "online_mode": s.get_online_mode(),
            "world_reset_policy": matches!(s.get_world_reset_policy(), WorldResetPolicy::AlwaysReset),
        })),
        "seed": seed,
        "level_type": level_type,
    });
    println!("{}", serde_json::to_string(&output).expect("serializable"));
    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_profile_and_context_wire_together() {
        let ctx = BuildContext::new(BuildProfile::parse("dev"));
        assert!(ctx.profile().is_dev());
    }
}
