use sand_commands::TextComponent;
use sand_components::{DatapackComponent, Instrument, JukeboxSong, RawJson, ResourceLocation};
use serde_json::{Value, json};

/// Build the required non-text fields for instrument validation cases.
fn instrument() -> Instrument {
    Instrument::new(ResourceLocation::new("test", "horn").unwrap())
        .sound_event("minecraft:item.goat_horn.sound.0")
}

/// Build the required non-text fields for jukebox validation cases.
fn song() -> JukeboxSong {
    JukeboxSong::new(ResourceLocation::new("test", "song").unwrap())
        .sound_event("minecraft:music_disc.13")
        .song_length(178.0)
}

/// Exercise identical raw text through both resource export boundaries.
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
        (
            json!({"selector": "@s", "separator": 42}),
            "description.separator",
        ),
        (
            json!({"text": "x", "hover_event": {"action": "run_command", "command": "/say hi"}}),
            "description.hover_event.action",
        ),
        (
            json!({"text": "x", "click_event": {"action": "show_text", "value": "hint"}}),
            "description.click_event.action",
        ),
        (json!({"object": "atlas"}), "description.sprite"),
        (
            json!({"object": "atlas", "sprite": 42}),
            "description.sprite",
        ),
        (
            json!({"object": "player", "player": 42}),
            "description.player",
        ),
        (
            json!({"object": "player", "player": {"id": "123e4567-e89b-12d3-a456-426614174000"}}),
            "description.player.id",
        ),
        (
            json!({"object": "player", "player": {"id": "bad uuid"}}),
            "description.player.id",
        ),
        (
            json!({"object": "player", "player": "Steve", "hat": "yes"}),
            "description.hat",
        ),
        (
            json!({"object": "atlas", "sprite": "block/stone", "fallback": {"text": 42}}),
            "description.fallback.text",
        ),
        (
            json!({"text": "x", "shadow_color": "red"}),
            "description.shadow_color",
        ),
        (
            json!({"text": "x", "shadow_color": [1, 1, 1]}),
            "description.shadow_color",
        ),
        (
            json!({"text": "x", "shadow_color": [1, 1, "red", 1]}),
            "description.shadow_color",
        ),
        (
            json!({"text": "x", "shadow_color": 4294967296_u64}),
            "description.shadow_color",
        ),
        (
            json!({"text": "x", "shadow_color": -2147483649_i64}),
            "description.shadow_color",
        ),
        (
            json!({"text": "x", "shadow_color": [1e300, 0, 0, 1]}),
            "description.shadow_color",
        ),
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
        json!({"text":"x","click_event":{"action":"custom","id":"click","payload":{"x":1}}}),
        json!({"text":"x","click_event":{"action":"show_dialog","dialog":"minecraft:server_links"}}),
        json!({"text":"x","click_event":{"action":"show_dialog","dialog":{"type":"minecraft:notice","title":{"text":"Hello"}}}}),
        json!({"text":"x","click_event":{"action":"open_url","url":"HTTPS://example.com/café?q=%20#ok"}}),
        json!({"nbt": "foo", "entity": "Steve"}),
        json!({"nbt": "foo", "entity": "123e4567-e89b-12d3-a456-426614174000"}),
        json!({"selector": "Steve", "separator": {"text": ", "}}),
        json!({"nbt": "foo", "entity": "@s", "separator": 42}),
        json!({"object": "atlas", "sprite": "block/stone", "fallback": {"text": "Stone"}}),
        json!({"type": "object", "sprite": "minecraft:item/diamond"}),
        json!({"object": "player", "player": "Steve", "hat": false}),
        json!({"player": {"name": "Steve", "id": [306070887, -392490285, -1537850778, 337068032]}, "fallback": ["Player ", {"text": "head"}]}),
        json!({"player": {}}),
        json!({"text": "x", "shadow_color": -1}),
        json!({"text": "x", "shadow_color": 0}),
        json!({"text": "x", "shadow_color": -2147483648_i64}),
        json!({"text": "x", "shadow_color": 4294967295_u64}),
        json!({"text": "x", "shadow_color": [1, 0.5, 0, 1]}),
        json!({"text": "x", "shadow_color": [2, -1, 0, 1]}),
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

/// Literal player targets retain their typed identity through resource text validation.
#[test]
fn typed_named_selector_descriptions_are_valid() {
    instrument()
        .description(TextComponent::selector(
            sand_commands::Target::named_player("Steve").to_string(),
        ))
        .try_content()
        .unwrap();
}

/// Every recognized click action must carry its required codec payload.
#[test]
fn raw_click_actions_require_payloads() {
    for (event, field) in [
        (json!({"action":"custom"}), "id"),
        (json!({"action":"custom", "id":42}), "id"),
        (json!({"action":"custom", "id":"bad id"}), "id"),
        (json!({"action":"show_dialog"}), "dialog"),
        (json!({"action":"show_dialog", "dialog":42}), "dialog"),
        (json!({"action":"show_dialog", "dialog":"bad id"}), "dialog"),
        (json!({"action":"open_file"}), "path"),
        (json!({"action":"open_file", "path":42}), "path"),
        (json!({"action":"open_file", "path":"test.txt"}), "action"),
    ] {
        for component in described(json!({"text":"x", "click_event":event})) {
            let error = component.try_content().unwrap_err();
            assert!(
                error.to_string().contains(&format!("click_event.{field}")),
                "{error}"
            );
        }
    }
}

/// Raw identifiers preserve Minecraft's omitted-namespace shorthand.
#[test]
fn raw_hover_ids_accept_default_namespace() {
    for event in [
        json!({"action":"show_item", "id":"stone"}),
        json!({"action":"show_entity", "id":"zombie", "uuid":[0,0,0,1]}),
    ] {
        for component in described(json!({"text":"x", "hover_event":event})) {
            component.try_content().unwrap();
        }
    }
}

/// Typed and raw URLs reject invalid URI syntax before resource export.
#[test]
fn click_urls_reject_malformed_uri_syntax() {
    for url in [
        "https://example.com/%",
        "https://example.com/%GG",
        "https://[broken/path",
        "https://example.com/{bad}",
        "https://example.com/a b",
    ] {
        assert!(
            TextComponent::literal("x")
                .click_open_url(url)
                .try_to_json_value()
                .is_err(),
            "{url}"
        );
        for component in
            described(json!({"text":"x", "click_event":{"action":"open_url", "url":url}}))
        {
            assert!(component.try_content().is_err(), "{url}");
        }
    }
}
