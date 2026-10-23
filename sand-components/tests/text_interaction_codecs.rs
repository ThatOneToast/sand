use sand_commands::{CommandProfile, EntityHoverId, Text, TextComponent};
use sand_components::{DatapackComponent, Instrument, ResourceLocation};
use serde_json::{Value, json};

/// Validate typed text before comparing its serialized codec fields.
fn rendered(text: TextComponent) -> Value {
    text.try_to_json_value().unwrap()
}

/// Legacy interaction fields load but vanish during vanilla decoding.
#[test]
fn typed_click_actions_use_current_codec_keys() {
    for (text, expected) in [
        (
            Text::new("Run").click_run_command("/say hi"),
            json!({"action": "run_command", "command": "/say hi"}),
        ),
        (
            Text::new("Suggest").click_suggest_command("/tell @s "),
            json!({"action": "suggest_command", "command": "/tell @s "}),
        ),
        (
            Text::new("Visit").click_open_url("https://example.com"),
            json!({"action": "open_url", "url": "https://example.com"}),
        ),
        (
            Text::new("Copy").click_copy("value"),
            json!({"action": "copy_to_clipboard", "value": "value"}),
        ),
        (
            Text::new("Page").click_change_page(2),
            json!({"action": "change_page", "page": 2}),
        ),
    ] {
        let value = rendered(text);
        assert_eq!(value["click_event"], expected);
        assert!(value.get("clickEvent").is_none());
    }
}

/// Styled hover text must live under the current value member.
#[test]
fn typed_hover_text_uses_current_value_field() {
    let value = rendered(Text::new("Hover").hover_text(Text::new("Hint").gold()));
    assert_eq!(
        value["hover_event"],
        json!({"action": "show_text", "value": {"text": "Hint", "color": "gold"}})
    );
    assert!(value.get("hoverEvent").is_none());
}

/// Keep entity kind and UUID distinct while preserving a styled tooltip name.
#[test]
fn typed_hover_entity_uses_type_id_and_uuid_fields() {
    let id = EntityHoverId::parse("123e4567-e89b-12d3-a456-426614174000").unwrap();
    let entity_type = sand_components::EntityTypeId::minecraft("zombie").unwrap();
    let value = rendered(Text::new("Entity").hover_entity(entity_type, id, Text::new("Undead")));
    assert_eq!(
        value["hover_event"],
        json!({"action": "show_entity", "id": "minecraft:zombie", "uuid": "123e4567-e89b-12d3-a456-426614174000", "name": {"text": "Undead"}})
    );
}

/// Reject an invalid book page before producing resource or command output.
#[test]
fn page_zero_is_rejected_before_rendering() {
    assert!(
        Text::new("Page")
            .click_change_page(0)
            .try_to_json_value()
            .is_err()
    );
}

/// Identify the exact malformed member instead of rejecting an entire resource opaquely.
#[test]
fn malformed_current_interactions_have_located_errors() {
    for (value, field) in [
        (json!({"text": "x", "click_event": 42}), "text.click_event"),
        (json!({"text": "x", "hover_event": 42}), "text.hover_event"),
        (
            json!({"text": "x", "click_event": {}}),
            "text.click_event.action",
        ),
        (
            json!({"text": "x", "click_event": {"action": "run_command", "value": "/say old"}}),
            "text.click_event.command",
        ),
        (
            json!({"text": "x", "click_event": {"action": "change_page", "page": 0}}),
            "text.click_event.page",
        ),
        (
            json!({"text": "x", "hover_event": {"action": "show_text", "value": {"text": 42}}}),
            "text.hover_event.value.text",
        ),
        (
            json!({"text": "x", "hover_event": {"action": "show_entity", "id": "minecraft:zombie"}}),
            "text.hover_event.uuid",
        ),
    ] {
        let error =
            sand_commands::text::validate_json_text(&value, &CommandProfile::unprofiled(), "text")
                .unwrap_err();
        assert_eq!(error.field, field);
    }
}

/// Exercise both inclusive codec bounds and their immediate invalid neighbors.
#[test]
fn typed_interaction_numeric_bounds_match_vanilla() {
    for page in [1, i32::MAX as u32] {
        Text::new("Page")
            .click_change_page(page)
            .try_to_json_value()
            .unwrap();
    }
    for page in [0, i32::MAX as u32 + 1, u32::MAX] {
        assert!(
            Text::new("Page")
                .click_change_page(page)
                .try_to_json_value()
                .is_err()
        );
    }
    for count in [1, 99] {
        Text::new("Item")
            .hover_item_with_count("minecraft:stone", count)
            .try_to_json_value()
            .unwrap();
    }
    for count in [0, 100, u32::MAX] {
        assert!(
            Text::new("Item")
                .hover_item_with_count("minecraft:stone", count)
                .try_to_json_value()
                .is_err()
        );
    }
}

/// Empty and whitespace-only chat strings survive both typed and raw codec paths.
#[test]
fn command_clicks_allow_empty_chat_strings_without_allowing_controls() {
    for command in ["", " ", "/say hello"] {
        for text in [
            Text::new("Run").click_run_command(command),
            Text::new("Suggest").click_suggest_command(command),
        ] {
            let json = text.try_to_json_value().unwrap();
            assert_eq!(json["click_event"]["command"], command);
            sand_commands::text::validate_json_text(&json, &CommandProfile::unprofiled(), "text")
                .unwrap();
            Instrument::new(ResourceLocation::new("test", "empty_command").unwrap())
                .sound_event(sand_components::SoundEventDefinition::reference(
                    "minecraft:item.goat_horn.sound.0"
                        .parse::<sand_components::SoundEventId>()
                        .unwrap(),
                ))
                .description(text)
                .try_content()
                .unwrap();
        }
    }
    for command in ["\n", "\t", "§", "\u{7f}"] {
        assert!(
            Text::new("Suggest")
                .click_suggest_command(command)
                .try_to_json_value()
                .is_err()
        );
    }
}
