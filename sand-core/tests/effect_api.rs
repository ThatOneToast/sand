use sand_core::cmd::{self, Selector};
use sand_core::{EffectId, PotionContents, PotionId, Range, StatusEffectInstance, Ticks};
use serde_json::json;

#[test]
fn typed_effect_commands_use_public_core_api() {
    assert_eq!(
        cmd::effect_give(Selector::self_(), EffectId::Speed)
            .duration(Ticks::seconds(10))
            .amplifier(1)
            .particles(false)
            .to_string(),
        "effect give @s minecraft:speed 10 1 true"
    );
    assert_eq!(cmd::effect_clear(Selector::self_()), "effect clear @s");
    assert_eq!(
        cmd::effect_clear_effect(Selector::self_(), EffectId::Regeneration),
        "effect clear @s minecraft:regeneration"
    );
}

#[test]
fn typed_effect_data_uses_public_core_api() {
    let custom = EffectId::custom("mymod:arcane_burn").unwrap();
    assert_eq!(custom.to_string(), "mymod:arcane_burn");

    let contents = PotionContents::new()
        .potion(PotionId::Swiftness)
        .effect(StatusEffectInstance::new(custom).duration(Ticks::new(40)));

    assert_eq!(
        serde_json::to_value(contents).unwrap(),
        json!({
            "potion": "minecraft:swiftness",
            "custom_effects": [{
                "id": "mymod:arcane_burn",
                "duration": 40,
                "amplifier": 0
            }]
        })
    );
}

#[test]
fn effect_predicate_uses_public_core_api() {
    let pred = sand_core::EffectPredicate::has(EffectId::Speed)
        .amplifier(Range::exact(1))
        .duration(Range::at_least(200));

    assert_eq!(
        serde_json::to_value(pred).unwrap(),
        json!({
            "minecraft:speed": {
                "amplifier": 1,
                "duration": {"min": 200}
            }
        })
    );
}
