use sand_components::{
    Biome, BiomeAdditionsSound, BiomeAmbientSounds, BiomeEffects, BiomeMoodSound,
    DatapackComponent, RawJson, SoundEventDefinition, SoundEventId,
};
use serde_json::{Value, json};

fn biome() -> Biome {
    Biome::new("test:audio".parse().unwrap(), BiomeEffects::new(0, 0, 0, 0))
}
fn reference() -> SoundEventDefinition {
    SoundEventDefinition::reference(SoundEventId::minecraft("ambient.cave").unwrap())
}
fn inline(range: Option<f32>) -> SoundEventDefinition {
    SoundEventDefinition::inline("test:wind".parse().unwrap(), range)
}
fn audio(value: &Biome) -> Value {
    value.validate().unwrap();
    value.to_json()["attributes"]["minecraft:audio/ambient_sounds"].clone()
}

#[test]
fn typed_audio_moves_to_native_attribute_with_shared_holders() {
    let value = biome().ambient_sounds(
        BiomeAmbientSounds::new()
            .loop_sound(reference())
            .mood_sound(BiomeMoodSound::new(
                inline(Some(-1.0)),
                i32::MIN,
                i32::MAX,
                -2.0,
            ))
            .additions_sound([
                BiomeAdditionsSound::new(reference(), 2.0),
                BiomeAdditionsSound::new(inline(None), -1.0),
            ]),
    );
    assert_eq!(
        audio(&value),
        json!({"loop":"minecraft:ambient.cave","mood":{"sound":{"sound_id":"test:wind","range":-1.0},"tick_delay":i32::MIN,"block_search_extent":i32::MAX,"offset":-2.0},"additions":[{"sound":"minecraft:ambient.cave","tick_chance":2.0},{"sound":{"sound_id":"test:wind"},"tick_chance":-1.0}]})
    );
    for key in ["ambient_sound", "mood_sound", "additions_sound"] {
        assert!(value.to_json()["effects"].get(key).is_none());
    }
    assert!(biome().to_json().get("attributes").is_none());
    assert_eq!(
        audio(&biome().ambient_sounds(BiomeAmbientSounds::new())),
        json!({"additions":[]})
    );
}

#[test]
fn raw_compact_additions_and_extensions_are_preserved_and_replaced() {
    let raw = json!({"loop":{"sound_id":"custom:wind","range":0,"extension":42},"additions":{"sound":"minecraft:ambient.cave","tick_chance":4},"extension":{"nested":true}});
    let value = biome()
        .ambient_sounds(BiomeAmbientSounds::new())
        .raw_ambient_sounds(RawJson::new(raw.clone()));
    assert_eq!(audio(&value), raw);
    assert_eq!(
        audio(&value.ambient_sounds(BiomeAmbientSounds::new())),
        json!({"additions":[]})
    );
}

#[test]
fn nonfinite_typed_fields_rejected_with_owner_and_nested_path() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for (sounds, path) in [
            (
                BiomeAmbientSounds::new().mood_sound(BiomeMoodSound::new(reference(), 1, 1, bad)),
                "mood.offset",
            ),
            (
                BiomeAmbientSounds::new()
                    .additions_sound([BiomeAdditionsSound::new(reference(), bad)]),
                "additions[0].tick_chance",
            ),
        ] {
            let error = biome()
                .ambient_sounds(sounds)
                .try_content()
                .unwrap_err()
                .to_string();
            for expected in [
                "test:audio",
                "worldgen/biome",
                "attributes.minecraft:audio/ambient_sounds",
                path,
                "finite",
            ] {
                assert!(error.contains(expected), "{error}");
            }
        }
    }
    for sounds in [
        BiomeAmbientSounds::new().loop_sound(inline(Some(f32::INFINITY))),
        BiomeAmbientSounds::new().mood_sound(BiomeMoodSound::new(
            inline(Some(f32::NAN)),
            1,
            1,
            1.0,
        )),
        BiomeAmbientSounds::new().additions_sound([BiomeAdditionsSound::new(
            inline(Some(f32::NEG_INFINITY)),
            0.1,
        )]),
    ] {
        assert!(biome().ambient_sounds(sounds).try_content().is_err());
    }
}

#[test]
fn malformed_raw_fields_fail_before_export() {
    for (raw, field) in [
        (json!([]), "ambient_sounds"),
        (json!({"loop":null}), "loop"),
        (json!({"loop":{"sound_id":"BAD"}}), "sound_id"),
        (json!({"loop":{"sound_id":"test:x","range":1e100}}), "range"),
        (json!({"mood":{}}), "sound"),
        (
            json!({"mood":{"sound":"minecraft:ambient.cave","tick_delay":2147483648i64,"block_search_extent":1,"offset":0}}),
            "tick_delay",
        ),
        (
            json!({"mood":{"sound":"minecraft:ambient.cave","tick_delay":1,"block_search_extent":1.5,"offset":0}}),
            "block_search_extent",
        ),
        (
            json!({"mood":{"sound":"minecraft:ambient.cave","tick_delay":1,"block_search_extent":1,"offset":null}}),
            "offset",
        ),
        (
            json!({"additions":[{"sound":"minecraft:ambient.cave"}]}),
            "additions[0].tick_chance",
        ),
        (json!({"additions":null}), "additions"),
        (json!({"mood_sound":{}}), "mood_sound"),
    ] {
        let error = biome()
            .raw_ambient_sounds(RawJson::new(raw))
            .try_content()
            .unwrap_err()
            .to_string();
        assert!(
            error.contains(field) && error.contains("test:audio"),
            "{error}"
        );
    }
}

#[test]
fn individual_setters_replace_values_and_reject_enclosing_maps() {
    let value = biome().ambient_sounds(
        BiomeAmbientSounds::new()
            .loop_sound(inline(None))
            .loop_sound(reference())
            .mood_sound(BiomeMoodSound::new(inline(None), 1, 1, 1.0))
            .mood_sound(BiomeMoodSound::new(reference(), 6000, 8, 2.0))
            .additions_sound([BiomeAdditionsSound::new(reference(), 1.0)])
            .additions_sound([]),
    );
    assert_eq!(
        audio(&value),
        json!({"loop":"minecraft:ambient.cave","mood":{"sound":"minecraft:ambient.cave","tick_delay":6000,"block_search_extent":8,"offset":2.0},"additions":[]})
    );
    assert_eq!(
        audio(&biome().raw_ambient_sounds(RawJson::new(json!({})))),
        json!({})
    );
    for raw in [
        json!({"attributes":{"minecraft:audio/ambient_sounds":{}}}),
        json!({"effects":{"mood_sound":{}}}),
        json!({"minecraft:audio/ambient_sounds":{}}),
    ] {
        assert!(
            biome()
                .raw_ambient_sounds(RawJson::new(raw))
                .try_content()
                .is_err()
        );
    }
}
