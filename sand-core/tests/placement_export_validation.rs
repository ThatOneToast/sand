//! Invalid typed worldgen input must fail the real component export boundary.
use sand::prelude::*;
use sand_core::{ComponentFactory, IntoDatapack};

inventory::submit! {
    ComponentFactory {
        owner: module_path!(),
        make: || PlacedFeature::new(
            ResourceLocation::new("invalid_placement", "ore").unwrap(),
            ConfiguredFeatureId::minecraft("ore_iron").unwrap(),
        ).placement([
            PlacementModifier::InSquare,
            PlacementModifier::Count(4097),
        ]).into_datapack(),
    }
}

#[test]
fn invalid_count_fails_export_with_resource_and_index() {
    let error = sand_core::try_export_components_json("invalid_placement")
        .expect_err("invalid placement must not be exported")
        .to_string();
    assert!(error.contains("invalid_placement:ore"), "{error}");
    assert!(error.contains("placement[1].count"), "{error}");
    assert!(error.contains("0..=4096"), "{error}");
}
