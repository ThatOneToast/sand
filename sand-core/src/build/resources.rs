//! Lowering a validated [`super::sand_build::SandBuild`] into concrete
//! datapack output records.
//!
//! The record shape here intentionally matches
//! `sand_core::compiler::export::records::ComponentRecord`'s wire format
//! (`namespace`/`dir`/`path`/`ext`/`content_type`/`content`) so `sand-cli`
//! can write both kinds of output through the same file-writing code path.

use sand_macros::api;
use serde::Serialize;

use super::dimension::DimensionSlot;
use super::sand_build::SandBuild;
use super::world::WeatherConfig;

/// One generated datapack file. See the module docs for why this mirrors
/// `ComponentRecord`'s shape.
///
/// 🌍 World (datapack) — every `WorldResource` this module produces is
/// written under `data/<namespace>/...` and ships with the exported
/// datapack.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::WorldResource",
    module = "sand::build",
    summary = "WorldResource is one generated datapack file produced by lowering a SandBuild's World.",
    context = "sand-cli writes each WorldResource under data/<namespace>/... using the same writer as ordinary component records.",
    minecraft = "Every WorldResource ships inside the exported datapack (dimension JSON, the generated world-init function, or its load-tag contribution).",
    use_when = ["Inspecting or testing what a World lowers to before sand-cli writes it"],
    avoid_when = ["Representing server-only settings; those live in ServerConfig and are never a WorldResource"],
    fields(namespace = "The resource's namespace, e.g. \"my_pack\" or \"minecraft\" for a tag contribution.", dir = "The component directory, e.g. \"dimension\" or \"function\".", path = "The resource location path within its directory.", ext = "The file extension without the dot, e.g. \"json\" or \"mcfunction\".", content_type = "How to interpret content; currently always \"text\".", content = "The generated file's text content."),
    example = "let resources = lower_world(\"my_pack\", &built);"
)]
pub struct WorldResource {
    pub namespace: String,
    pub dir: String,
    pub path: String,
    pub ext: &'static str,
    pub content_type: &'static str,
    pub content: String,
}

/// Lowers a [`SandBuild`]'s [`super::world::World`] into the datapack
/// resources it produces: one `dimension` JSON per configured dimension,
/// plus a generated `__sand_world_init` function (spawn/border/gamerules/
/// time/weather commands) and its `minecraft:load` tag contribution.
///
/// Returns an empty vec if no [`super::world::World`] was configured.
#[api(
    registry = sand_api_contract,
    path = "sand::build::lower_world",
    module = "sand::build",
    summary = "Lowers a SandBuild's World into the concrete datapack resources it produces.",
    context = "Called by the generated sand_build_world binary's main (via run_and_print) and directly in tests.",
    minecraft = "Produces one dimension JSON per configured dimension plus a generated world-init function and its minecraft:load tag contribution, when the World configures spawn/border/gamerules/time/weather.",
    use_when = ["Turning a validated SandBuild into files sand-cli can write"],
    avoid_when = ["Validating a SandBuild; call SandBuild::validate first"],
    params(namespace = "The project's pack namespace.", build = "The build whose World should be lowered."),
    returns = "The generated WorldResource list; empty if no World was configured.",
    example = "let resources = lower_world(\"my_pack\", &SandBuild::new());"
)]
pub fn lower(namespace: &str, build: &SandBuild) -> Vec<WorldResource> {
    let mut out = Vec::new();
    let Some(world) = &build.world else {
        return out;
    };

    for dim in world.dimensions.entries() {
        let rl = dim.resource_location();
        out.push(WorldResource {
            namespace: rl.namespace().to_string(),
            dir: "dimension".to_string(),
            path: rl.path().to_string(),
            ext: "json",
            content_type: "text",
            content: serde_json::to_string_pretty(&dim.to_json()).unwrap(),
        });
        if let DimensionSlot::Custom(_) = dim.slot {
            // Custom dimension_type resources are the project's own
            // responsibility (referenced, not generated) unless they used
            // `DimensionType::Custom`, in which case Sand still does not
            // author the file's contents — only the reference — matching
            // the "advanced worldgen stays hand-authored" non-goal.
        }
    }

    let mut commands = Vec::new();
    if let Some(spawn) = &world.spawn {
        commands.push(format!(
            "setworldspawn {} {} {} {}",
            spawn.x, spawn.y, spawn.z, spawn.yaw
        ));
        if let Some(platform) = &spawn.platform {
            let r = platform.radius as i32;
            commands.push(format!(
                "fill {} {} {} {} {} {} {}",
                spawn.x - r,
                spawn.y - 1,
                spawn.z - r,
                spawn.x + r,
                spawn.y - 1,
                spawn.z + r,
                platform.block
            ));
        }
    }
    if let Some(border) = &world.border {
        commands.push(format!(
            "worldborder center {} {}",
            border.center_x, border.center_z
        ));
        commands.push(format!("worldborder set {}", border.diameter));
        commands.push(format!(
            "worldborder damage amount {}",
            border.damage_per_block
        ));
        commands.push(format!(
            "worldborder warning distance {}",
            border.warning_distance
        ));
        commands.push(format!(
            "worldborder warning time {}",
            border.warning_time
        ));
    }
    for (name, value) in &world.gamerules {
        commands.push(format!("gamerule {name} {value}"));
    }
    if let Some(time) = &world.time {
        commands.push(format!("time set {}", time.ticks));
        if time.freeze {
            commands.push("gamerule doDaylightCycle false".to_string());
        }
    }
    if let Some(weather) = &world.weather {
        let cmd = match weather {
            WeatherConfig::Clear => "weather clear",
            WeatherConfig::Rain => "weather rain",
            WeatherConfig::Thunder => "weather thunder",
        };
        commands.push(cmd.to_string());
    }

    if !commands.is_empty() {
        out.push(WorldResource {
            namespace: namespace.to_string(),
            dir: "function".to_string(),
            path: "__sand_world_init".to_string(),
            ext: "mcfunction",
            content_type: "text",
            content: commands.join("\n") + "\n",
        });
        // `minecraft:load` tag contribution — `sand-cli` merges this with
        // any tag file the ordinary component exporter also produced for
        // `minecraft:load`, rather than overwriting it. See
        // `sand-cli/src/build/worldbuild.rs`.
        out.push(WorldResource {
            namespace: "minecraft".to_string(),
            dir: "tags/function".to_string(),
            path: "load".to_string(),
            ext: "json",
            content_type: "text",
            content: serde_json::to_string_pretty(&serde_json::json!({
                "values": [format!("{namespace}:__sand_world_init")]
            }))
            .unwrap(),
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::dimension::{Dimension, DimensionSlot, DimensionType, Dimensions};
    use crate::build::generator::{FlatGenerator, FlatLayer, Generator};
    use crate::build::world::{Spawn, World, WorldBorder};
    use sand_components::resource_location::ResourceLocation;

    #[test]
    fn empty_build_lowers_to_nothing() {
        let build = SandBuild::new();
        assert!(lower("pack", &build).is_empty());
    }

    #[test]
    fn dimension_lowers_to_a_dimension_json_resource() {
        let build = SandBuild::new().world(World::new().dimensions(Dimensions::new().with(
            Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
                Generator::Flat(FlatGenerator::new(vec![FlatLayer::new(
                    ResourceLocation::new("minecraft", "stone").unwrap(),
                    64,
                )])),
            ),
        )));
        let resources = lower("pack", &build);
        let dim = resources
            .iter()
            .find(|r| r.dir == "dimension")
            .expect("dimension resource present");
        assert_eq!(dim.namespace, "minecraft");
        assert_eq!(dim.path, "overworld");
        assert_eq!(dim.ext, "json");
    }

    #[test]
    fn spawn_border_and_gamerules_lower_to_the_init_function_and_load_tag() {
        let build = SandBuild::new().world(
            World::new()
                .spawn(Spawn::at(0, 100, 0))
                .border(WorldBorder::diameter(2000.0))
                .gamerule("keepInventory", "true"),
        );
        let resources = lower("pack", &build);
        let func = resources
            .iter()
            .find(|r| r.dir == "function" && r.path == "__sand_world_init")
            .expect("init function present");
        assert!(func.content.contains("setworldspawn 0 100 0"));
        assert!(func.content.contains("worldborder set 2000"));
        assert!(func.content.contains("gamerule keepInventory true"));

        let tag = resources
            .iter()
            .find(|r| r.namespace == "minecraft" && r.dir == "tags/function")
            .expect("load tag present");
        assert!(tag.content.contains("pack:__sand_world_init"));
    }

    #[test]
    fn dev_and_release_worlds_produce_different_dimension_json() {
        let dev = SandBuild::new().world(World::new().dimensions(Dimensions::new().with(
            Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
                Generator::Flat(FlatGenerator::new(vec![FlatLayer::new(
                    ResourceLocation::new("minecraft", "grass_block").unwrap(),
                    1,
                )])),
            ),
        )));
        let release = SandBuild::new().world(World::new().dimensions(Dimensions::new().with(
            Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
                Generator::Noise(crate::build::generator::NoiseGenerator::vanilla(
                    crate::build::generator::VanillaNoiseSettings::Overworld,
                )),
            ),
        )));

        let dev_dim = lower("pack", &dev)
            .into_iter()
            .find(|r| r.dir == "dimension")
            .unwrap();
        let release_dim = lower("pack", &release)
            .into_iter()
            .find(|r| r.dir == "dimension")
            .unwrap();
        assert_ne!(dev_dim.content, release_dim.content);
        assert!(dev_dim.content.contains("minecraft:flat"));
        assert!(release_dim.content.contains("minecraft:noise"));
    }
}
