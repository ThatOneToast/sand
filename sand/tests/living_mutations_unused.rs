//! Merely constructing living operations must not install runtime machinery.
use sand::prelude::*;

#[test]
fn discarded_living_actions_add_no_resources_or_runtime_cost() {
    let before = sand::advanced::try_export_pack_json("unused", "26.2").unwrap();
    let living = EntityContext::<PlayerKind>::default().living();
    let _ = living.damage(1.0, DamageKind::Generic);
    let _ = living.give_effect(EffectId::Speed).seconds(5);
    let _ = living.clear_effect(EffectId::Speed);
    let _ = living.clear_effects();
    assert_eq!(
        before,
        sand::advanced::try_export_pack_json("unused", "26.2").unwrap()
    );
}
