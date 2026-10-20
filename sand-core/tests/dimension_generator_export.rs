use sand::component::{
    BiomeSource, Dimension, FlatGenerator, FlatLayer, Generator, NoiseGenerator,
    VanillaNoiseSettings,
};
use sand::registration::DatapackComponent;
use sand::registry::{BiomeId, BlockId, DimensionTypeId};
use sand::{ResourceLocation, component::RawJson};
use sand_core::{ComponentFactory, IntoDatapack};
use serde_json::json;

fn rl(path: &str) -> ResourceLocation {
    ResourceLocation::new("shared_generators", path).unwrap()
}
fn checkerboard() -> Generator {
    Generator::Noise(
        NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld).biome_source(
            BiomeSource::Checkerboard {
                biomes: vec![
                    BiomeId::minecraft("plains").unwrap(),
                    BiomeId::minecraft("desert").unwrap(),
                ],
                scale: 2,
            },
        ),
    )
}
inventory::submit! {
    ComponentFactory {
        owner: module_path!(),
        make: || Dimension::new(rl("checkerboard"), DimensionTypeId::minecraft("overworld").unwrap(), checkerboard()).into_datapack(),
    }
}

#[test]
fn component_and_build_paths_export_identical_generator_resources() {
    use sand::build::{
        Dimension as BuildDimension, DimensionSlot, DimensionType, Dimensions, SandBuild, World,
        lower_world,
    };
    for generator in [
        Generator::Void,
        Generator::Debug,
        Generator::Flat(
            FlatGenerator::new(vec![FlatLayer::new(
                BlockId::minecraft("stone").unwrap(),
                5,
            )])
            .with_structures(true),
        ),
        Generator::Noise(NoiseGenerator::vanilla(VanillaNoiseSettings::Nether)),
        Generator::Noise(NoiseGenerator::vanilla(VanillaNoiseSettings::End)),
        checkerboard(),
        Generator::Raw(RawJson::new(json!({"type": "test:custom", "custom": true}))),
    ] {
        let component = Dimension::new(
            rl("world"),
            DimensionTypeId::minecraft("overworld").unwrap(),
            generator.clone(),
        );
        component.validate().unwrap();
        let build = SandBuild::new().world(
            World::new().dimensions(
                Dimensions::new().with(
                    BuildDimension::new(
                        DimensionSlot::Custom(rl("world")),
                        DimensionType::Overworld,
                    )
                    .generator(generator),
                ),
            ),
        );
        build.validate().unwrap();
        let records = lower_world("shared_generators", &build);
        let record = records.iter().find(|r| r.dir == "dimension").unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&record.content).unwrap(),
            component.to_json()
        );
    }
}

#[test]
fn registered_checkerboard_exports_ordered_biomes_deterministically() {
    let export = || sand_core::try_export_components_json("shared_generators").unwrap();
    assert_eq!(export(), export());
    let records = sand_core::try_export_components("shared_generators").unwrap();
    let record = records
        .iter()
        .find(|r| r.dir == "dimension" && r.path == "checkerboard")
        .unwrap();
    let actual: serde_json::Value = serde_json::from_str(&record.content).unwrap();
    assert_eq!(
        actual["generator"]["biome_source"],
        json!({"type": "minecraft:checkerboard", "biomes": ["minecraft:plains", "minecraft:desert"], "scale": 2})
    );
}

#[test]
fn tall_custom_dimension_accepts_flat_layers_above_overworld_height() {
    use sand::build::{
        Dimension as BuildDimension, DimensionSlot, DimensionType, Dimensions, SandBuild, World,
        lower_world,
    };
    let generator = Generator::Flat(FlatGenerator::new(vec![FlatLayer::new(
        BlockId::minecraft("stone").unwrap(),
        512,
    )]));
    let dimension_type = sand::component::DimensionType::overworld_like(rl("tall_type"))
        .height(512)
        .logical_height(512);
    dimension_type.validate().unwrap();
    let component = Dimension::new(
        rl("tall_world"),
        DimensionTypeId::custom(rl("tall_type")),
        generator.clone(),
    );
    component.validate().unwrap();
    let build = SandBuild::new().world(
        World::new().dimensions(
            Dimensions::new().with(
                BuildDimension::new(
                    DimensionSlot::Custom(rl("tall_world")),
                    DimensionType::Custom(rl("tall_type")),
                )
                .generator(generator),
            ),
        ),
    );
    build.validate().unwrap();
    let records = lower_world("shared_generators", &build);
    let record = records.iter().find(|r| r.dir == "dimension").unwrap();
    let actual: serde_json::Value = serde_json::from_str(&record.content).unwrap();
    assert_eq!(actual, component.to_json());
    assert_eq!(actual["generator"]["settings"]["layers"][0]["height"], 512);
}

#[test]
fn build_checkerboard_checks_each_vanilla_biome_reference() {
    use sand::build::{
        Dimension as BuildDimension, DimensionSlot, DimensionType, Dimensions, SandBuild, World,
    };
    let generator = Generator::Noise(
        NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld).biome_source(
            BiomeSource::Checkerboard {
                biomes: vec![
                    BiomeId::minecraft("plains").unwrap(),
                    BiomeId::minecraft("dessert").unwrap(),
                ],
                scale: 2,
            },
        ),
    );
    let build = SandBuild::new().world(
        World::new().dimensions(
            Dimensions::new().with(
                BuildDimension::new(DimensionSlot::Custom(rl("world")), DimensionType::Overworld)
                    .generator(generator),
            ),
        ),
    );
    let errors = build.validate().unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("dessert"));
    assert!(errors[0].message.contains("not a known vanilla biome"));
}

#[test]
fn component_and_build_validation_reject_the_same_invalid_shapes() {
    use sand::build::{
        Dimension as BuildDimension, DimensionSlot, DimensionType, Dimensions, SandBuild, World,
    };
    for generator in [
        Generator::Flat(FlatGenerator::new(vec![FlatLayer::new(
            BlockId::minecraft("stone").unwrap(),
            0,
        )])),
        Generator::Noise(
            NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld).biome_source(
                BiomeSource::Checkerboard {
                    biomes: vec![],
                    scale: 2,
                },
            ),
        ),
        Generator::Raw(RawJson::new(json!("test:bare_generator_reference"))),
    ] {
        let component = Dimension::new(
            rl("invalid"),
            DimensionTypeId::minecraft("overworld").unwrap(),
            generator.clone(),
        );
        let sand::component::SandError::ComponentValidation { field, message, .. } =
            component.validate().unwrap_err()
        else {
            panic!("expected contextual validation error")
        };
        let build = SandBuild::new().world(
            World::new().dimensions(
                Dimensions::new().with(
                    BuildDimension::new(
                        DimensionSlot::Custom(rl("invalid")),
                        DimensionType::Overworld,
                    )
                    .generator(generator),
                ),
            ),
        );
        let errors = build.validate().unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].location,
            format!("Dimension[shared_generators:invalid].{field}")
        );
        assert_eq!(errors[0].message, message);
    }
}
