mod support;
use support::*;

#[test]
fn native_damage_types_emit_resources_and_preserve_attribution() {
    let pack = compile(&analyze_one(include_str!("fixtures/damage_types.sand")));
    let data: serde_json::Value =
        serde_json::from_slice(&pack.resources["data/language_probe/damage_type/thorn.json"])
            .unwrap();
    assert_eq!(data["message_id"], "scout_thorn");
    assert_eq!(data["effects"], "thorns");
    assert_eq!(data["scaling"], "never");
    assert_eq!(data["death_message_type"], "default");
    assert!(pack.resources.iter().any(|(path, commands)| {
        path.ends_with(".mcfunction")
            && String::from_utf8_lossy(commands)
                .contains("damage @s $(amount) language_probe:thorn by ")
    }));
}

#[test]
fn damage_type_checks_ids_and_typed_options() {
    for (body, expected) in [
        (
            "damage_type Thorn { effects: DamageEffects::Hurt }",
            "`damage_type` needs `message_id`",
        ),
        (
            "damage_type Thorn { message_id: \"thorn\", exhaustion: -1 }",
            "exhaustion must be finite and non-negative",
        ),
        (
            "damage_type Thorn { message_id: \"thorn\", scaling: \"demo:never\" }",
            "expected a `DamageScaling::...` value",
        ),
        (
            "fn run() { damage(1, type = DamageType::Missing); }",
            "no damage type named",
        ),
    ] {
        let source = format!("#![pack(namespace = \"damage_example\")]\n{body}");
        let issues = errors(&analyze_one(&source));
        assert!(
            issues.iter().any(|issue| issue.contains(expected)),
            "{issues:?}"
        );
    }
}
