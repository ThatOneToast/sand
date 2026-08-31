//! Typed build-time world configuration for Trailforge (issue #317).
//!
//! Demonstrates the canonical dev/release split: a fast flat world with a
//! spawn platform for local iteration, full vanilla noise generation for
//! the shipped pack. See the book's "Typed World & Server Configuration"
//! chapters (23-34) for the full API this exercises.
//!
//! Compiled by `sand-cli` as the `sand_build_world` binary (see the
//! `[[bin]]` entry in Cargo.toml) and run during `sand build`/`sand run`.

use sand::build::{
    BuildContext, Difficulty, Dimension, DimensionSlot, DimensionType, Dimensions, FlatGenerator,
    FlatLayer, Generator, NoiseGenerator, SandBuild, Seed, ServerConfig, Spawn, VanillaNoiseSettings,
    World, WorldResetPolicy,
};
use sand::ResourceLocation;

fn build(ctx: &BuildContext) -> SandBuild {
    let (overworld, seed, reset_policy) = if ctx.profile().is_dev() || ctx.profile().is_test() {
        // dev/test: fast flat world with a spawn platform, deterministic
        // seed, and a fresh world every `sand run` for clean iteration.
        let dimension = Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld)
            .generator(Generator::Flat(FlatGenerator::new(vec![
                FlatLayer::new(ResourceLocation::new("minecraft", "bedrock").unwrap(), 1),
                FlatLayer::new(ResourceLocation::new("minecraft", "stone").unwrap(), 32),
                FlatLayer::new(ResourceLocation::new("minecraft", "grass_block").unwrap(), 1),
            ])));
        (dimension, Seed::Fixed(1_337), WorldResetPolicy::AlwaysReset)
    } else {
        // release (and any other profile): full vanilla noise generation,
        // random seed, and the local world persists across `sand run`s.
        let dimension = Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld)
            .generator(Generator::Noise(NoiseGenerator::vanilla(
                VanillaNoiseSettings::Overworld,
            )));
        (dimension, Seed::Random, WorldResetPolicy::Keep)
    };

    SandBuild::new()
        .world(
            World::new()
                .seed(seed)
                .spawn(
                    Spawn::at(0, 65, 0)
                        .platform(ResourceLocation::new("minecraft", "stone").unwrap(), 4),
                )
                .gamerule("doDaylightCycle", "true")
                .dimensions(Dimensions::new().with(overworld)),
        )
        .server(
            ServerConfig::new()
                .view_distance(if ctx.profile().is_dev() { 6 } else { 10 })
                .difficulty(Difficulty::Normal)
                .world_reset_policy(reset_policy),
        )
}

fn main() {
    let profile = sand::build::BuildProfile::parse(
        &std::env::var("SAND_BUILD_PROFILE").unwrap_or_else(|_| "dev".to_string()),
    );
    let mc_version =
        std::env::var("SAND_EXPORT_MC_VERSION").unwrap_or_else(|_| "latest".to_string());
    let ctx = BuildContext::new(profile).with_mc_version(mc_version);
    sand::build::run_and_print("trail", ctx, build);
}
