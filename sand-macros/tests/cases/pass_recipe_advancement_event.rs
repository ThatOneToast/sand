// Canonical recipe: advancement-backed event with a typed guard and a rewarded
// #[function] that applies a status effect. Exercises the full
// AdvancementEvent → Event<T> → #[event] pipeline.
use sand_core::event::trigger::ConsumeItemTrigger;
use sand_core::prelude::*;
use sand_macros::{event, function};

static STRENGTH_STACKS: ScoreVar<i32> = ScoreVar::new("str_stacks");

pub struct AteChorusFruitEvent;

impl AdvancementEvent for AteChorusFruitEvent {
    type Trigger = ConsumeItemTrigger;

    fn trigger() -> Self::Trigger {
        ConsumeItemTrigger::new().item(ItemPredicate::id("minecraft:chorus_fruit"))
    }

    fn guard() -> Option<Condition> {
        // Only trigger while the player has fewer than 5 strength stacks.
        Some(STRENGTH_STACKS.of("@s").lt(5))
    }
}

#[function]
pub fn apply_strength_buff() {
    cmd::effect_give(Selector::self_(), EffectId::Strength)
        .seconds(30)
        .amplifier(0);
    cmd::tellraw(Selector::self_(), Text::new("Strength granted!").red());
}

#[event]
pub fn on_ate_chorus_fruit(event: Event<AteChorusFruitEvent>) {
    STRENGTH_STACKS.add(event.player(), 1);
    cmd::call(apply_strength_buff);
}

fn main() {
    let commands = on_ate_chorus_fruit();
    assert!(
        commands
            .iter()
            .any(|c| c.contains("scoreboard players add") && c.contains("str_stacks")),
        "expected str_stacks increment; got: {commands:?}"
    );
    assert!(
        commands
            .iter()
            .any(|c| c.contains("function") && c.contains("apply_strength_buff")),
        "expected call to apply_strength_buff; got: {commands:?}"
    );

    // The guard condition should check that stacks < 5 (matches ..4).
    let mut found_guard = false;
    for descriptor in inventory::iter::<sand_core::EventDescriptor>() {
        if descriptor.path == "on_ate_chorus_fruit" {
            if let sand_core::EventDispatch::Advancement { guard, .. } = descriptor.dispatch {
                let guard_fn = guard.expect("guard must be registered");
                let condition = guard_fn().expect("guard should return Some");
                let guard_cmds = condition.execute_commands(true, "return 0");
                assert!(
                    guard_cmds
                        .iter()
                        .any(|c| c.contains("score @s str_stacks matches ..4")),
                    "guard should enforce str_stacks < 5 (matches ..4); got: {guard_cmds:?}"
                );
                found_guard = true;
            }
        }
    }
    assert!(found_guard, "on_ate_chorus_fruit event descriptor not found");
}
