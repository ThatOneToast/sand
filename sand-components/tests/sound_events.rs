use sand_commands::TextComponent;
use sand_components::{
    DatapackComponent, Instrument, JukeboxSong, RawJson, SoundEventDefinition, SoundEventId,
};
use serde_json::{Value, json};

fn consumers(sound: SoundEventDefinition) -> Vec<Box<dyn DatapackComponent>> {
    vec![
        Box::new(
            Instrument::new("test:horn".parse().unwrap())
                .sound_event(sound.clone())
                .description(TextComponent::literal("Horn")),
        ),
        Box::new(
            JukeboxSong::new("test:song".parse().unwrap())
                .sound_event(sound)
                .song_length(10.0)
                .description(TextComponent::literal("Song")),
        ),
    ]
}

fn assert_sound(sound: SoundEventDefinition, expected: Value) {
    for resource in consumers(sound) {
        resource.try_content().unwrap();
        assert_eq!(resource.to_json()["sound_event"], expected);
    }
}

#[test]
fn reference_and_inline_shapes_are_shared_and_distinct() {
    assert_sound(
        SoundEventDefinition::reference(SoundEventId::minecraft("block.note_block.harp").unwrap()),
        json!("minecraft:block.note_block.harp"),
    );
    assert_sound(
        SoundEventDefinition::reference("custom:registered".parse::<SoundEventId>().unwrap()),
        json!("custom:registered"),
    );
    assert_sound(
        SoundEventDefinition::inline("custom:asset".parse().unwrap(), None),
        json!({"sound_id":"custom:asset"}),
    );
    for range in [32.0, 0.0, -1.0, f32::MIN, f32::MAX] {
        assert_sound(
            SoundEventDefinition::inline("custom:asset".parse().unwrap(), Some(range)),
            json!({"sound_id":"custom:asset", "range":range}),
        );
    }
}

#[test]
fn inline_range_is_independent_of_instrument_range() {
    let resource = Instrument::new("test:horn".parse().unwrap())
        .sound_event(SoundEventDefinition::inline(
            "custom:asset".parse().unwrap(),
            Some(-1.0),
        ))
        .range(256.0)
        .description(TextComponent::literal("Horn"));
    resource.try_content().unwrap();
    assert_eq!(resource.to_json()["range"], 256.0);
    assert_eq!(resource.to_json()["sound_event"]["range"], -1.0);
    assert!(resource.range(-1.0).try_content().is_err());
}

#[test]
fn non_finite_typed_ranges_fail_before_serialization_with_resource_and_field() {
    for range in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for resource in consumers(SoundEventDefinition::inline(
            "test:asset".parse().unwrap(),
            Some(range),
        )) {
            let error = resource.try_content().unwrap_err().to_string();
            assert!(
                error.contains(&resource.resource_location().to_string()),
                "{error}"
            );
            assert!(error.contains(resource.component_dir()), "{error}");
            assert!(error.contains("[field: sound_event.range]"), "{error}");
            assert!(error.contains("finite"), "{error}");
        }
    }
}

#[test]
fn raw_known_shapes_are_validated_and_extensions_retained() {
    for value in [
        json!("minecraft:block.note_block.harp"),
        json!({"sound_id":"custom:asset", "extension":{"values":[1,true,null]}}),
        json!({"sound_id":"custom:asset", "range":-1.0, "extra":42}),
    ] {
        assert_sound(
            SoundEventDefinition::raw(RawJson::new(value.clone())),
            value,
        );
    }
    for (value, field) in [
        (json!(null), "sound_event"),
        (json!(42), "sound_event"),
        (json!([]), "sound_event"),
        (json!(""), "sound_event"),
        (json!("#minecraft:tag"), "sound_event"),
        (json!("invalid space"), "sound_event"),
        (json!({}), "sound_event.sound_id"),
        (json!({"sound_id":42}), "sound_event.sound_id"),
        (json!({"sound_id":"#test:tag"}), "sound_event.sound_id"),
        (json!({"sound_id":"invalid space"}), "sound_event.sound_id"),
        (
            json!({"sound_id":"test:asset", "range":null}),
            "sound_event.range",
        ),
        (
            json!({"sound_id":"test:asset", "range":"32"}),
            "sound_event.range",
        ),
        (
            json!({"sound_id":"test:asset", "range":true}),
            "sound_event.range",
        ),
        (
            json!({"sound_id":"test:asset", "range":1e100}),
            "sound_event.range",
        ),
    ] {
        for resource in consumers(SoundEventDefinition::raw(RawJson::new(value.clone()))) {
            let error = resource.try_content().unwrap_err().to_string();
            assert!(
                error.contains(&format!("[field: {field}]")),
                "{value}: {error}"
            );
            assert!(
                error.contains(&resource.resource_location().to_string()),
                "{error}"
            );
        }
    }
}

#[test]
fn selecting_a_sound_replaces_the_entire_previous_definition() {
    let raw = SoundEventDefinition::raw(RawJson::new(json!({"sound_id":"test:old", "extra":1})));
    let typed = SoundEventDefinition::inline("test:new".parse().unwrap(), None);
    let horn = Instrument::new("test:horn".parse().unwrap())
        .sound_event(raw.clone())
        .sound_event(typed.clone())
        .description(TextComponent::literal("Horn"));
    let song = JukeboxSong::new("test:song".parse().unwrap())
        .sound_event(typed)
        .sound_event(raw)
        .song_length(1.0)
        .description(TextComponent::literal("Song"));
    horn.try_content().unwrap();
    song.try_content().unwrap();
    assert_eq!(
        horn.to_json()["sound_event"],
        json!({"sound_id":"test:new"})
    );
    assert_eq!(
        song.to_json()["sound_event"],
        json!({"sound_id":"test:old", "extra":1})
    );
}

#[test]
fn missing_sound_is_rejected_for_both_consumers() {
    let resources: Vec<Box<dyn DatapackComponent>> = vec![
        Box::new(
            Instrument::new("test:horn".parse().unwrap())
                .description(TextComponent::literal("Horn")),
        ),
        Box::new(
            JukeboxSong::new("test:song".parse().unwrap())
                .song_length(1.0)
                .description(TextComponent::literal("Song")),
        ),
    ];
    for resource in resources {
        let error = resource.try_content().unwrap_err().to_string();
        assert!(error.contains("[field: sound_event]"), "{error}");
        assert!(error.contains("required"), "{error}");
    }
}
