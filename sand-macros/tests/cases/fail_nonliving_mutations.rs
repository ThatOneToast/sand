use sand::prelude::*;

fn main() {
    let entity = EntityContext::<AnyEntity>::default();
    entity.living().damage(1.0, DamageKind::Generic);
    let marker = EntityContext::<MarkerKind>::default();
    marker.living().give_effect(EffectId::Speed);
}
