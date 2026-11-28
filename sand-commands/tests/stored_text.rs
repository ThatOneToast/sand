use sand_commands::{CommandProfile, DataTarget, NbtValue, TextComponent};

#[test]
fn typed_strings_escape_every_control_character_without_command_injection() {
    let value = "\0\n\r\t\u{001b}\u{007f}\u{0085}\"\\$(uuid) 🌍";
    let literal = NbtValue::String(value.into());
    let expected = r#""\u0000\u000a\u000d\u0009\u001b\u007f\u0085\"\\$(uuid) 🌍""#;
    assert_eq!(literal.to_string(), expected);
    for version in ["26.2", "26.3"] {
        let rendered = DataTarget::storage("demo:state")
            .path("text")
            .set(literal.clone())
            .try_render(&CommandProfile::new(version, false))
            .unwrap();
        assert!(!rendered.chars().any(char::is_control));
        assert!(rendered.ends_with(expected));
    }
    let encoded = serde_json::to_string(&literal).unwrap();
    assert_eq!(serde_json::from_str::<NbtValue>(&encoded).unwrap(), literal);
    assert!(serde_json::to_string(&NbtValue::List(vec![NbtValue::raw("[]")])).is_err());
    assert!(serde_json::from_str::<NbtValue>(r#"{"type":"raw","value":"[]"}"#).is_err());
}

#[test]
fn typed_nbt_text_stays_plain_and_validates_its_readable_reference() {
    let text = TextComponent::nbt(
        DataTarget::storage("demo:state").path("characters[]"),
        TextComponent::literal(""),
    );
    let json = text.try_to_json_value().unwrap();
    assert_eq!(json["storage"], "demo:state");
    assert_eq!(json["nbt"], "characters[]");
    assert_eq!(json["interpret"], false);
    assert_eq!(json["separator"]["text"], "");
    let invalid = TextComponent::nbt(
        DataTarget::storage("bad storage").path("characters[]"),
        TextComponent::literal(""),
    );
    assert!(invalid.try_to_json_value().is_err());
}
