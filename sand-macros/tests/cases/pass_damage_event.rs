use sand_core::prelude::*;
use sand_macros::event;

pub struct HurtEvent;

impl AdvancementEvent for HurtEvent {
    type Trigger = sand_core::AdvancementTrigger;

    fn trigger() -> Self::Trigger {
        sand_core::AdvancementTrigger::EntityHurtPlayer {
            entity: None,
            damage: None,
        }
    }
}

impl DamageAdvancementEvent for HurtEvent {}

#[event]
pub fn on_hurt(event: DamageEvent<HurtEvent>) {
    event
        .reflect_damage()
        .to(EntityTargets::nearby(5.0).excluding_players().excluding_self())
        .amount(DamageAmount::fixed(4.0))
        .damage_type(DamageKind::Generic)
        .run();
}

fn main() {
    let commands = on_hurt();
    assert_eq!(
        commands,
        vec![
            "execute at @s as @e[distance=0.1..5,type=!minecraft:player] run damage @s 4 minecraft:generic"
        ]
    );
}
