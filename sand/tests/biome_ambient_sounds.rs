//! Supported facade export proves namespace isolation and native attribute placement.
use sand::component::{
    Biome, BiomeAdditionsSound, BiomeAmbientSounds, BiomeEffects, BiomeMoodSound,
    SoundEventDefinition,
};
use sand_core::serde_json::{self, json};

#[sand::datapack_component]
fn ambient_biome() -> Biome {
    let reference = SoundEventDefinition::reference(
        sand::registry::SoundEventId::minecraft("ambient.cave").unwrap(),
    );
    Biome::new(
        "__sand_local:ambient".parse().unwrap(),
        BiomeEffects::new(0, 0, 0, 0),
    )
    .ambient_sounds(
        BiomeAmbientSounds::new()
            .loop_sound(reference.clone())
            .mood_sound(BiomeMoodSound::new(reference, 6000, 8, 2.0))
            .additions_sound([BiomeAdditionsSound::new(
                SoundEventDefinition::inline("assets:wind".parse().unwrap(), Some(-1.0)),
                2.0,
            )]),
    )
}

#[test]
fn audio_attribute_exports_deterministically_without_namespace_leaks() {
    let export = |namespace| sand::advanced::try_export_pack_json(namespace, "26.2").unwrap();
    let first = export("audio_a");
    let second = export("audio_b");
    assert_eq!(first, export("audio_a"));
    for (namespace, pack) in [("audio_a", first), ("audio_b", second)] {
        let pack: serde_json::Value = serde_json::from_str(&pack).unwrap();
        let records: Vec<_> = pack["components"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["dir"] == "worldgen/biome")
            .collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0]["namespace"], namespace);
        assert_eq!(records[0]["path"], "ambient");
        let biome: serde_json::Value =
            serde_json::from_str(records[0]["content"].as_str().unwrap()).unwrap();
        assert_eq!(
            biome["attributes"]["minecraft:audio/ambient_sounds"],
            json!({"loop":"minecraft:ambient.cave","mood":{"sound":"minecraft:ambient.cave","tick_delay":6000,"block_search_extent":8,"offset":2.0},"additions":[{"sound":{"sound_id":"assets:wind","range":-1.0},"tick_chance":2.0}]})
        );
        assert!(biome["effects"].get("ambient_sound").is_none());
    }
}
