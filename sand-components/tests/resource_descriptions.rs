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

/// Translation arguments preserve numeric and boolean values without permitting primitive roots.
#[test]
fn raw_translation_arguments_preserve_primitives() {
    let value = json!({"translate":"key", "with":[7, 1.5, true, false, {"text":"suffix"}]});
    for component in described(value.clone()) {
        component.try_content().unwrap();
        assert_eq!(component.to_json()["description"], value);
    }
    for value in [
        json!(7),
        json!(true),
        json!({"translate":"key", "with":[null]}),
    ] {
        for component in described(value) {
            assert!(component.try_content().is_err());
        }
    }
}

/// Every raw resource identifier accepts the vanilla default namespace.
#[test]
fn raw_storage_and_font_ids_accept_default_namespace() {
    for value in [
        json!({"nbt":"foo", "storage":"data"}),
        json!({"text":"x", "font":"default"}),
    ] {
        for component in described(value.clone()) {
            component.try_content().unwrap();
            assert_eq!(component.to_json()["description"], value);
        }
    }
}

/// Explicit vanilla discriminators cannot silently fall back to inferred content.
#[test]
fn raw_unknown_vanilla_discriminators_are_rejected() {
    for value in [
        json!({"type":"literal", "text":"x"}),
        json!({"type":"minecraft:literal", "text":"x"}),
        json!({"type":"minecraft:text", "text":"x"}),
        json!({"object":"unknown", "sprite":"block/stone"}),
        json!({"nbt":"foo", "source":"unknown", "storage":"data"}),
    ] {
        for component in described(value) {
            assert!(component.try_content().is_err());
        }
    }
}

/// Inline dialogs validate known variant requirements and button payloads.
#[test]
fn raw_inline_dialog_variants_require_valid_fields() {
    for dialog in [
        json!({"type":"multi_action", "title":"Menu"}),
        json!({"type":"multi_action", "title":"Menu", "actions":[]}),
        json!({"type":"multi_action", "title":"Menu", "actions":[{}]}),
        json!({"type":"multi_action", "title":"Menu", "actions":[{"label":"Go", "width":0}]}),
        json!({"type":"confirmation", "title":"Confirm", "yes":{"label":"Yes"}}),
        json!({"type":"dialog_list", "title":"Dialogs"}),
        json!({"type":"server_links", "title":"Links", "columns":0}),
        json!({"type":"notice", "title":"Info", "pause":"yes"}),
        json!({"type":"notice", "title":"Info", "after_action":"none"}),
        json!({"type":"notice", "title":"Info", "action":{"label":"Go", "action":{"type":"run_command"}}}),
        json!({"type":"unknown", "title":"Info"}),
    ] {
        for component in
            described(json!({"text":"x", "click_event":{"action":"show_dialog", "dialog":dialog}}))
        {
            assert!(component.try_content().is_err(), "{dialog}");
        }
    }
}

/// All vanilla dialog variants preserve valid nested buttons, bodies and controls.
#[test]
fn raw_inline_dialog_variants_round_trip() {
    for dialog in [
        json!({"type":"confirmation", "title":"Confirm", "yes":{"label":"Yes"}, "no":{"label":"No"}}),
        json!({"type":"multi_action", "title":"Menu", "actions":[{"label":"Go", "tooltip":"Hint", "width":1024, "action":{"type":"run_command", "command":"/say hi"}}], "columns":1}),
        json!({"type":"server_links", "title":"Links", "button_width":1, "exit_action":{"label":"Done"}}),
        json!({"type":"dialog_list", "title":"Dialogs", "dialogs":[{"type":"notice", "title":"Nested"}]}),
        json!({"type":"notice", "title":"Info", "pause":false, "after_action":"none", "external_title":"Open", "body":[
            {"type":"plain_message", "contents":"Message", "width":1024},
            {"type":"item", "item":{"id":"stone", "count":2}, "description":{"contents":"Stone", "width":100}, "show_tooltip":false, "width":256, "height":1}
        ], "inputs":[
            {"key":"flag", "type":"boolean", "label":"Flag", "initial":true},
            {"key":"text", "type":"text", "label":"Text", "initial":"Hi", "multiline":{"height":512, "max_lines":2}},
            {"key":"choice", "type":"single_option", "label":"Choice", "options":["first", {"id":"second", "display":"Second", "initial":true}]},
            {"key":"range", "type":"number_range", "label":"Range", "start":10, "end":0, "initial":5, "step":0.5}
        ]}),
        json!({"type":"example:custom_dialog", "extension":"preserved"}),
    ] {
        let value = json!({"text":"x", "click_event":{"action":"show_dialog", "dialog":dialog}});
        for resource in described(value.clone()) {
            resource.try_content().unwrap();
            assert_eq!(resource.to_json()["description"], value);
        }
    }
    for fields in [
        json!({"body":{"type":"plain_message"}}),
        json!({"body":{"type":"item", "item":{"id":"stone", "count":0}}}),
        json!({"body":{"type":"item", "item":"stone", "height":257}}),
        json!({"inputs":42}),
        json!({"inputs":[{"key":"x", "type":"boolean", "label":"Flag", "initial":"yes"}]}),
        json!({"inputs":[{"key":"x", "type":"text", "label":"Text", "max_length":0}]}),
        json!({"inputs":[{"key":"x", "type":"single_option", "label":"Choice", "options":[]}]}),
        json!({"inputs":[{"key":"x", "type":"single_option", "label":"Choice", "options":[{"id":"a", "initial":true}, {"id":"b", "initial":true}]}]}),
        json!({"inputs":[{"key":"x", "type":"number_range", "label":"Range", "start":0, "end":10, "initial":11}]}),
        json!({"inputs":[{"key":"x", "type":"number_range", "label":"Range", "start":0, "end":10, "step":0}]}),
    ] {
        let mut dialog = json!({"type":"notice", "title":"Info"});
        dialog
            .as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        for resource in
            described(json!({"text":"x", "click_event":{"action":"show_dialog", "dialog":dialog}}))
        {
            assert!(resource.try_content().is_err(), "{dialog}");
        }
    }
}

/// Input keys follow Java's BMP letter/decimal-digit grammar, including empty names.
#[test]
fn raw_dialog_input_keys_match_vanilla_variables() {
    for (key, valid) in [
        ("player_1", true),
        ("", true),
        ("λ١_中文", true),
        ("bad-key", false),
        ("bad key", false),
        ("Ⅳ", false),
        ("½", false),
        ("a\u{301}", false),
        ("𐐀", false),
    ] {
        let value = json!({"text":"x", "click_event":{"action":"show_dialog", "dialog":{
            "type":"notice", "title":"Input", "inputs":[{"key":key,"type":"boolean","label":"Flag"}]
        }}});
        for resource in described(value.clone()) {
            let result = resource.try_content();
            assert_eq!(result.is_ok(), valid, "{key:?}: {result:?}");
            if valid {
                assert_eq!(resource.to_json()["description"], value);
            } else {
                assert_eq!(
                    match result.unwrap_err() {
                        sand_components::SandError::ComponentValidation { field, .. } => field,
                        error => panic!("unexpected error: {error}"),
                    },
                    "description.click_event.dialog.inputs[0].key"
                );
            }
        }
    }
}

/// Dynamic command templates require complete variables with the same input-key grammar.
#[test]
fn raw_dialog_command_templates_validate_variables() {
    for (template, valid) in [
        ("say $(player_1)", true),
        ("say $(λ١_中文) $()", true),
        ("say $literal $(x) $", true),
        ("say hi", false),
        ("say $(bad-key)", false),
        ("say $(x", false),
        ("say $(x) $(", false),
        ("say $(Ⅳ)", false),
        ("say $(𐐀)", false),
    ] {
        let value = json!({"text":"x", "click_event":{"action":"show_dialog", "dialog":{
            "type":"notice", "title":"Action", "action":{"label":"Go", "action":{
                "type":"dynamic/run_command", "template":template
            }}
        }}});
        for resource in described(value.clone()) {
            let result = resource.try_content();
            assert_eq!(result.is_ok(), valid, "{template:?}: {result:?}");
            if valid {
                assert_eq!(resource.to_json()["description"], value);
            } else {
                assert_eq!(
                    match result.unwrap_err() {
                        sand_components::SandError::ComponentValidation { field, .. } => field,
                        error => panic!("unexpected error: {error}"),
                    },
                    "description.click_event.dialog.action.action.template"
                );
            }
        }
    }
}

/// Only non-Minecraft namespaces identify unsupported extension content.
#[test]
fn raw_extension_components_exclude_reserved_minecraft_keys() {
    for (value, valid) in [
        (json!({"minecraft:bogus":true}), false),
        (json!({"example:future":true}), true),
        (json!({"text":"x", "minecraft:metadata":true}), true),
        // Empty brackets select every list element; they are not an invalid index.
        (json!({"nbt":"foo[]", "entity":"@s"}), true),
    ] {
        for resource in described(value.clone()) {
            assert_eq!(resource.try_content().is_ok(), valid, "{value}");
            if valid {
                assert_eq!(resource.to_json()["description"], value);
            }
        }
    }
}

/// Typed text and resource descriptions share raw text's default-namespace semantics.
#[test]
fn typed_description_identifiers_accept_default_namespace() {
    let text = TextComponent::literal("x")
        .font("default")
        .hover_item("stone");
    let value = text.try_to_json_value().unwrap();
    assert_eq!(value["font"], "default");
    assert_eq!(value["hover_event"]["id"], "stone");
    let resources: Vec<Box<dyn DatapackComponent>> = vec![
        Box::new(instrument().description(text.clone())),
        Box::new(song().description(text)),
    ];
    for resource in resources {
        resource.try_content().unwrap();
        assert_eq!(resource.to_json()["description"], value);
    }
}

/// Unknown vanilla interaction actions fail while explicit raw extension actions remain available.
#[test]
fn raw_interaction_actions_require_known_vanilla_or_explicit_extension_names() {
    for family in ["click_event", "hover_event"] {
        for (action, valid) in [
            ("bogus", false),
            ("minecraft:bogus", false),
            ("minecraft:run_command", false),
            ("example:future_action", true),
        ] {
            let mut value = json!({"text":"x"});
            value[family] = json!({"action":action, "extension":"preserved"});
            for resource in described(value.clone()) {
                assert_eq!(resource.try_content().is_ok(), valid, "{value}");
                if valid {
                    assert_eq!(resource.to_json()["description"], value);
                }
            }
        }
    }
}

/// Explicit raw extension names still require valid resource-identifier syntax.
#[test]
fn raw_extension_identifiers_validate_their_namespace_and_path() {
    for id in ["bad!:future", ":future", "example:", "example:bad?path"] {
        for value in [
            json!({"text":"x", "click_event":{"action":id}}),
            json!({"text":"x", "hover_event":{"action":id}}),
            json!({"type":id, "payload":true}),
            json!({"object":id}),
            json!({"nbt":"foo", "source":id}),
            Value::Object(serde_json::Map::from_iter([(id.into(), json!(true))])),
        ] {
            for resource in described(value.clone()) {
                assert!(resource.try_content().is_err(), "{value}");
            }
        }
    }
    for value in [
        json!({"type":"example:future", "payload":true}),
        json!({"object":"example:future"}),
        json!({"nbt":"foo", "source":"example:future"}),
        json!({"example:future":true}),
    ] {
        for resource in described(value.clone()) {
            resource.try_content().unwrap();
            assert_eq!(resource.to_json()["description"], value);
        }
    }
}

/// Item-hover patches validate set and removal identifiers without rewriting raw values.
#[test]
fn raw_hover_item_component_patch_keys_are_valid_identifiers() {
    for key in [
        "bad id",
        "bad!:name",
        "minecraft:",
        "!",
        "!!minecraft:custom_name",
    ] {
        let patch = serde_json::Map::from_iter([(key.into(), json!({}))]);
        for resource in described(json!({"text":"x", "hover_event":{
            "action":"show_item", "id":"stone", "components":patch
        }})) {
            assert!(resource.try_content().is_err(), "{key}");
        }
    }
    let value = json!({"text":"x", "hover_event":{
        "action":"show_item", "id":"stone", "components":{
            "custom_name":{"text":"Patched"}, "!minecraft:lore":{}
        }
    }});
    for resource in described(value.clone()) {
        resource.try_content().unwrap();
        assert_eq!(resource.to_json()["description"], value);
    }
}

/// Parse path nodes instead of accepting any balanced bracket contents.
#[test]
fn resource_nbt_paths_use_the_current_path_grammar() {
    for path in [
        "foo[abc]",
        "foo[+1]",
        "foo[1.0]",
        "foo[2147483648]",
        "foo..bar",
        "foo{a:}.bar",
        "foo[0]{a:1}",
        r#""a\n""#,
        r#""""#,
    ] {
        for component in described(json!({"nbt":path,"entity":"@s"})) {
            let error = component.try_content().unwrap_err();
            assert!(
                error.to_string().contains("description.nbt"),
                "{path}: {error}"
            );
        }
    }
    for path in [
        "foo[]",
        "foo[-1]",
        "foo.",
        "foo.[0]",
        r#""foo..bar""#,
        "'foo.bar'",
        "foo{a:1}.bar",
        "{a:1}.bar",
        "foo[{a:1}]",
        "foo[{}]",
        "foo{a:bool(1)}",
        "",
    ] {
        for component in described(json!({"nbt":path,"entity":"@s"})) {
            component
                .try_content()
                .unwrap_or_else(|e| panic!("{path}: {e}"));
        }
    }
}

/// Record-form profiles enforce Java string bounds and the 16-property maximum.
#[test]
fn player_profile_record_limits_match_the_codec() {
    let property = json!({"name":"textures", "value":"x"});
    let mut invalid = vec![json!(vec![property.clone(); 17])];
    for (field, size) in [("name", 65), ("value", 32768), ("signature", 1025)] {
        let mut record = property.clone();
        record[field] = json!("x".repeat(size));
        invalid.push(json!([record]));
    }
    invalid.push(json!([{"name":"😀".repeat(33),"value":"x"}]));
    for properties in invalid {
        for component in described(json!({"object":"player","player":{"properties":properties}})) {
            let error = component.try_content().unwrap_err();
            assert!(
                error.to_string().contains("description.player.properties"),
                "{error}"
            );
        }
    }
    for properties in [
        json!(vec![property; 16]),
        json!([{"name":"😀".repeat(32),"value":"x".repeat(32767),"signature":"x".repeat(1024)}]),
    ] {
        for component in described(json!({"object":"player","player":{"properties":properties}})) {
            component.try_content().unwrap();
        }
    }
}

/// Legacy property maps limit distinct keys but retain unbounded string lists.
#[test]
fn player_profile_map_limits_do_not_inherit_record_limits() {
    let properties: serde_json::Map<String, Value> =
        (0..17).map(|i| (i.to_string(), json!(["x"]))).collect();
    for component in described(json!({"object":"player","player":{"properties":properties}})) {
        assert!(component.try_content().is_err());
    }
    let mut properties = serde_json::Map::new();
    properties.insert("x".repeat(65), json!(vec!["x".repeat(32768); 17]));
    for component in described(json!({"object":"player","player":{"properties":properties}})) {
        component.try_content().unwrap();
    }
}
