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

/// Reject malformed vanilla text before either registry resource can be published.
#[test]
fn malformed_description_is_rejected_before_export() {
    for (value, field) in [
        (json!(42), "description"),
        (json!({}), "description"),
        (json!({"bold": true}), "description"),
        (
            json!({"text": "x", "extra": [{"color": "gold"}]}),
            "description.extra[0]",
        ),
        (json!([]), "description"),
        (json!({"text": "x", "extra": []}), "description.extra"),
        (json!({"text": 42}), "description.text"),
        (
            json!({"text": "x", "click_event": {"action": "run_command", "command": "say\nhi"}}),
            "description.click_event.command",
        ),
        (
            json!({"text": "x", "insertion": 42}),
            "description.insertion",
        ),
        (
            json!({"text": "x", "click_event": 42}),
            "description.click_event",
        ),
        (
            json!({"text": "x", "hover_event": 42}),
            "description.hover_event",
        ),
        (
            json!({"text": "x", "click_event": {}}),
            "description.click_event.action",
        ),
        (
            json!({"text": "x", "hover_event": {"action": "show_text"}}),
            "description.hover_event.value",
        ),
        (
            json!({"text": "x", "click_event": {"action": "run_command", "value": "/say old"}}),
            "description.click_event.command",
        ),
        (
            json!({"text": "x", "click_event": {"action": "change_page", "page": 0}}),
            "description.click_event.page",
        ),
        (
            json!({"text": "x", "click_event": {"action": "open_url", "url": "bad url"}}),
            "description.click_event.url",
        ),
        (
            json!({"text": "x", "hover_event": {"action": "show_text", "value": {"text": 42}}}),
            "description.hover_event.value.text",
        ),
        (
            json!({"text": "x", "hover_event": {"action": "show_entity", "id": "minecraft:zombie"}}),
            "description.hover_event.uuid",
        ),
        (
            json!({"text": "x", "hover_event": {"action": "show_item", "id": "minecraft:stone", "count": 100}}),
            "description.hover_event.count",
        ),
        (json!({"translate": false}), "description.translate"),
        (
            json!({"translate": "song.name", "fallback": 42}),
            "description.fallback",
        ),
        (json!({"keybind": []}), "description.keybind"),
        (json!({"selector": {}}), "description.selector"),
        (json!({"nbt": 1}), "description.nbt"),
        (json!({"nbt": "foo"}), "description.source"),
        (json!({"nbt": "", "entity": "@s"}), "description.nbt"),
        (json!({"nbt": "foo[", "entity": "@s"}), "description.nbt"),
        (
            json!({"nbt": "foo", "source": "block", "entity": "@s"}),
            "description.source",
        ),
        (
            json!({"nbt": "foo", "entity": "@s", "interpret": true, "plain": true}),
            "description",
        ),
        (json!({"text": "x", "type": "score"}), "description.score"),
        (json!({"text": "x", "type": 42}), "description.type"),
        (json!({"nbt": "foo", "entity": 42}), "description.entity"),
        (json!({"nbt": "foo", "storage": []}), "description.storage"),
        (json!({"nbt": "foo", "block": false}), "description.block"),
        (
            json!({"nbt": "foo", "storage": "bad id"}),
            "description.storage",
        ),
        (
            json!({"nbt": "foo", "block": "bad coordinates"}),
            "description.block",
        ),
        (json!({"nbt": "foo", "block": "^ ~ ~"}), "description.block"),
        (
            json!({"nbt": "foo", "block": "1.5 0 0"}),
            "description.block",
        ),
        (
            json!({"nbt": "foo", "entity": "@invalid"}),
            "description.entity",
        ),
        (
            json!({"nbt": "foo", "entity": "@s", "storage": "test:data"}),
            "description.source",
        ),
        (json!({"text": "x", "color": 42}), "description.color"),
        (json!({"text": "x", "font": false}), "description.font"),
        (json!({"text": "x", "bold": "yes"}), "description.bold"),
        (json!({"text": "x", "with": {}}), "description.with"),
        (json!({"text": "x", "extra": "bad"}), "description.extra"),
        (json!({"score": []}), "description.score"),
        (
            json!({"score": {"objective": "points"}}),
            "description.score.name",
        ),
        (
            json!({"score": {"name": "@s", "objective": false}}),
            "description.score.objective",
        ),
        (
            json!({"text": "x", "extra": [{"text": 42}]}),
            "description.extra[0].text",
        ),
        (json!(true), "description"),
        (Value::Null, "description"),
        (
            json!({"score": {"name": "@s", "objective": "bad name"}}),
            "description.score.objective",
        ),
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

/// Both codecs require descriptions even when every sound field is valid.
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

/// Compare complete JSON so a legacy duration key cannot silently return.
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

/// Zero is a valid silent comparator value in the vanilla song codec.
#[test]
fn jukebox_comparator_zero_is_valid() {
    song()
        .description(TextComponent::literal("Silent comparator"))
        .comparator_output(0)
        .try_content()
        .unwrap();
}

/// Raw compatibility preserves supported shapes and explicit extension payloads.
#[test]
fn explicit_text_shapes_round_trip() {
    for value in [
        json!("Name"),
        json!({"nbt": "foo", "source": "mymod:custom", "mymod:data": "value"}),
        json!({"nbt": "foo", "entity": "@s"}),
        json!({"nbt": "foo", "entity": "@s", "interpret": "yes"}),
        json!({"nbt": "foo", "storage": "test:data"}),
        json!({"nbt": "foo", "block": "~ ~0.5 -2"}),
        json!({"nbt": "foo", "block": "^ ^1.5 ^"}),
        json!({"translate": "song.name", "fallback": "Song"}),
        json!({"type": "mymod:custom", "value": "Name"}),
        json!({"mymod:custom": "Name"}),
        json!({"translate": "resource.test.name", "with": []}),
        json!({"text": "x", "insertion": "name"}),
        json!({"text": "x", "click_event": {"action": "run_command", "command": "/say hello"}}),
        json!({"text": "x", "click_event": {"action": "suggest_command", "command": "/tell @s "}}),
        json!({"text": "x", "click_event": {"action": "open_url", "url": "https://example.com"}}),
        json!({"text": "x", "click_event": {"action": "copy_to_clipboard", "value": "copy"}}),
        json!({"text": "x", "click_event": {"action": "change_page", "page": 1}}),
        json!({"text": "x", "hover_event": {"action": "show_text", "value": {"text": "hint"}}}),
        json!({"text": "x", "hover_event": {"action": "show_item", "id": "minecraft:stone", "count": 99}}),
        json!({"text": "x", "hover_event": {"action": "show_entity", "id": "minecraft:zombie", "uuid": [0, 1, -2, 3], "name": {"text": "zombie"}}}),
        json!({"text": "x", "hover_event": {"action": "show_entity", "id": "minecraft:zombie", "uuid": "123e4567-e89b-12d3-a456-426614174000"}}),
        json!({"text": "x", "clickEvent": 42, "hoverEvent": 42}),
        json!({"score": {"name": "@s", "objective": "points"}}),
        json!({"text": "Name", "italic": false, "font": "minecraft:default", "color": "gold"}),
        json!(["Name ", {"text": "suffix", "bold": true}]),
        json!({"translate": "resource.test.name", "mymod:style": true}),
    ] {
        for resource in described(value.clone()) {
            resource.try_content().unwrap();
            assert_eq!(resource.to_json()["description"], value);
        }
    }
}

/// Resource text uses the same styled model and located errors as command text.
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
        assert!(error.contains("description.style.font"), "{error}");
    }
}
