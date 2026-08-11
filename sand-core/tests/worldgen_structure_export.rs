use sand_core::ComponentFactory;
use sand_core::prelude::*;

fn town_center_pool() -> TemplatePool {
    TemplatePool::new(
        ResourceLocation::new("worldgen_structure_export", "town_centers").unwrap(),
        TemplatePoolId::empty(),
        [PoolEntry::new(
            PoolElement::single(
                StructureTemplateId::minecraft("village/plains/town_centers/1").unwrap(),
            ),
            1,
        )],
    )
}

fn outpost_structure() -> Structure {
    Structure::jigsaw(
        ResourceLocation::new("worldgen_structure_export", "outpost").unwrap(),
        TemplatePoolId::custom(
            ResourceLocation::new("worldgen_structure_export", "town_centers").unwrap(),
        ),
        BiomeSelector::Tag(TagId::minecraft("has_structure/village_plains").unwrap()),
    )
}

fn village_structure_set() -> StructureSet {
    StructureSet::random_spread(
        ResourceLocation::new("worldgen_structure_export", "villages").unwrap(),
        StructureId::minecraft("village_plains").unwrap(),
        34,
        8,
        10_387_312,
    )
}

fn mossify_processors() -> ProcessorList {
    ProcessorList::new(
        ResourceLocation::new("worldgen_structure_export", "mossify").unwrap(),
        [Processor::BlockIgnore(vec![
            BlockId::minecraft("air").unwrap(),
        ])],
    )
}

inventory::submit! {
    ComponentFactory { make: || Box::new(town_center_pool()) }
}
inventory::submit! {
    ComponentFactory { make: || Box::new(outpost_structure()) }
}
inventory::submit! {
    ComponentFactory { make: || Box::new(village_structure_set()) }
}
inventory::submit! {
    ComponentFactory { make: || Box::new(mossify_processors()) }
}

fn find_record<'a>(
    records: &'a [sand_core::ComponentRecord],
    dir: &str,
    path: &str,
) -> &'a sand_core::ComponentRecord {
    records
        .iter()
        .find(|record| {
            record.namespace == "worldgen_structure_export"
                && record.dir == dir
                && record.path == path
        })
        .unwrap_or_else(|| panic!("expected {dir}/{path} to be exported"))
}

#[test]
fn structure_generation_registries_export_to_vanilla_worldgen_paths() {
    let records = sand_core::try_export_components("worldgen_structure_export")
        .expect("export should succeed");

    let pool = find_record(&records, "worldgen/template_pool", "town_centers");
    assert_eq!(pool.ext, "json");
    let pool_json: serde_json::Value = serde_json::from_str(&pool.content).unwrap();
    assert_eq!(pool_json["fallback"], "minecraft:empty");

    let structure = find_record(&records, "worldgen/structure", "outpost");
    let structure_json: serde_json::Value = serde_json::from_str(&structure.content).unwrap();
    assert_eq!(structure_json["type"], "minecraft:jigsaw");
    assert_eq!(
        structure_json["start_pool"],
        "worldgen_structure_export:town_centers"
    );

    let structure_set = find_record(&records, "worldgen/structure_set", "villages");
    let structure_set_json: serde_json::Value =
        serde_json::from_str(&structure_set.content).unwrap();
    assert_eq!(
        structure_set_json["placement"]["type"],
        "minecraft:random_spread"
    );

    let processors = find_record(&records, "worldgen/processor_list", "mossify");
    let processors_json: serde_json::Value = serde_json::from_str(&processors.content).unwrap();
    assert_eq!(
        processors_json["processors"][0]["processor_type"],
        "minecraft:block_ignore"
    );
}
