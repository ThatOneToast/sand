use sand_commands::TextComponent;
use sand_components::{DatapackComponent, Instrument, JukeboxSong, RawJson, ResourceLocation};
use serde_json::{Value, json};

fn instrument() -> Instrument {
    Instrument::new(ResourceLocation::new("test", "horn").unwrap())
        .sound_event("minecraft:item.goat_horn.sound.0")
}

fn song() -> JukeboxSong {
    JukeboxSong::new(ResourceLocation::new("test", "song").unwrap())
        .sound_event("minecraft:music_disc.13")
        .song_length(178.0)
}

fn described(value: Value) -> Vec<Box<dyn DatapackComponent>> {
    vec![
        Box::new(instrument().raw_description(RawJson::new(value.clone()))),
        Box::new(song().raw_description(RawJson::new(value))),
    ]
}

#[test]
fn malformed_description_is_rejected_before_export() {
    for (value, field) in [
        (json!(42), "description"),
        (json!(true), "description"),
        (Value::Null, "description"),
        (json!([false]), "description[0]"),
        (
            json!({"text": "name", "extra": [false]}),
            "description.extra[0]",
        ),
        (json!({"translate": "bad\nkey"}), "description.translate"),
        (
            json!({"text": "name", "font": "bad font"}),
            "description.font",
        ),
    ] {
        for resource in described(value) {
            let error = resource.try_content().unwrap_err().to_string();
            assert!(error.contains(resource.component_dir()), "{error}");
            assert!(error.contains(&format!("field: {field}")), "{error}");
        }
    }
}

#[test]
fn required_description_is_checked_before_export() {
    for resource in [
        Box::new(instrument()) as Box<dyn DatapackComponent>,
        Box::new(song()),
    ] {
        let error = resource.try_content().unwrap_err().to_string();
        assert!(error.contains("description"), "{error}");
    }
}

#[test]
fn jukebox_duration_uses_vanilla_field_name() {
    let resource = song().description(TextComponent::translate("song.test.theme"));
    resource.try_content().unwrap();
    assert_eq!(
        resource.to_json(),
        json!({
            "sound_event": "minecraft:music_disc.13",
            "length_in_seconds": 178.0,
            "comparator_output": 1,
            "description": {"translate": "song.test.theme"},
        })
    );
}

#[test]
fn jukebox_comparator_zero_is_valid() {
    song()
        .description(TextComponent::literal("Silent comparator"))
        .comparator_output(0)
        .try_content()
        .unwrap();
}

#[test]
fn explicit_text_shapes_round_trip() {
    for value in [
        json!("Name"),
        json!(["Name ", {"text": "suffix", "bold": true}]),
        json!({"translate": "resource.test.name", "mymod:style": true}),
    ] {
        for resource in described(value.clone()) {
            resource.try_content().unwrap();
            assert_eq!(resource.to_json()["description"], value);
        }
    }
}

#[test]
fn typed_descriptions_share_command_text_validation_and_json() {
    let text = TextComponent::translate("resource.test.name").color_hex("#abc123");
    let resources: Vec<Box<dyn DatapackComponent>> = vec![
        Box::new(instrument().description(text.clone())),
        Box::new(song().description(text)),
    ];
    for resource in resources {
        resource.try_content().unwrap();
        assert_eq!(
            resource.to_json()["description"],
            json!({
                "translate": "resource.test.name", "color": "#abc123",
            })
        );
    }
    let invalid = TextComponent::literal("Name").font("bad font");
    let resources: Vec<Box<dyn DatapackComponent>> = vec![
        Box::new(instrument().description(invalid.clone())),
        Box::new(song().description(invalid)),
    ];
    for resource in resources {
        let error = resource.try_content().unwrap_err().to_string();
        assert!(error.contains("description.font"), "{error}");
    }
}
