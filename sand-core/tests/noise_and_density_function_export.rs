use sand_core::ComponentFactory;
use sand_core::prelude::*;

fn ridges_noise() -> Noise {
    Noise::new(
        ResourceLocation::new("noise_density_export", "ridges").unwrap(),
        -7,
        [1.0, 1.0],
    )
}

fn ridge_density_function() -> DensityFunction {
    DensityFunction::new(
        ResourceLocation::new("noise_density_export", "ridge_density").unwrap(),
        DensityFunctionExpr::square(DensityFunctionExpr::noise(
            NoiseId::custom(ResourceLocation::new("noise_density_export", "ridges").unwrap()),
            1.0,
            1.0,
        )),
    )
}

inventory::submit! {
    ComponentFactory { make: || Box::new(ridges_noise()) }
}

inventory::submit! {
    ComponentFactory { make: || Box::new(ridge_density_function()) }
}

#[test]
fn noise_exports_to_the_worldgen_noise_directory() {
    let records =
        sand_core::try_export_components("noise_density_export").expect("export should succeed");
    let record = records
        .iter()
        .find(|record| {
            record.namespace == "noise_density_export"
                && record.dir == "worldgen/noise"
                && record.path == "ridges"
        })
        .expect("noise record should be exported");

    assert_eq!(record.ext, "json");
    let json: serde_json::Value = serde_json::from_str(&record.content).unwrap();
    assert_eq!(json["firstOctave"], -7);
    assert_eq!(json["amplitudes"], serde_json::json!([1.0, 1.0]));
}

#[test]
fn density_function_exports_to_the_worldgen_density_function_directory() {
    let records =
        sand_core::try_export_components("noise_density_export").expect("export should succeed");
    let record = records
        .iter()
        .find(|record| {
            record.namespace == "noise_density_export"
                && record.dir == "worldgen/density_function"
                && record.path == "ridge_density"
        })
        .expect("density function record should be exported");

    assert_eq!(record.ext, "json");
    let json: serde_json::Value = serde_json::from_str(&record.content).unwrap();
    assert_eq!(json["type"], "minecraft:square");
    assert_eq!(json["argument"]["noise"], "noise_density_export:ridges");
}
