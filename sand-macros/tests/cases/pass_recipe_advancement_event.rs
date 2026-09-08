// Canonical recipe: advancement-backed event with a typed guard and a rewarded
// #[function] that applies a status effect. Exercises the full
// AdvancementEvent → Event<T> → #[on_event] pipeline.
use sand::event::trigger::ConsumeItemTrigger;
use sand::prelude::*;

#[derive(State)]
#[state(namespace = "recipe", scope = player)]
#[allow(dead_code)]
struct StrengthState {
    #[state(default = 0, min = 0, max = 5)]
    stacks: Score,
}

fn strength() -> StrengthStateBound {
    StrengthState::on(EntityContext::<PlayerKind>::default())
}

pub struct AteChorusFruitEvent;

impl AdvancementEvent for AteChorusFruitEvent {
    type Trigger = ConsumeItemTrigger;

    fn trigger() -> Self::Trigger {
        ConsumeItemTrigger::new().item(ItemPredicate::id(
            ItemId::minecraft("chorus_fruit").unwrap(),
        ))
    }

    fn guard() -> Option<Condition> {
        // Only trigger while the player has fewer than 5 strength stacks.
        Some(strength().stacks.matches(..5).unwrap())
    }
}

#[function]
pub fn apply_strength_buff() {
    cmd::effect_give(Target::self_(), EffectId::Strength)
        .seconds(30)
        .amplifier(0);
    cmd::tellraw(Target::self_(), Text::new("Strength granted!").red());
}

#[on_event]
pub fn on_ate_chorus_fruit(event: Event<AteChorusFruitEvent>) {
    let _ = event;
    strength().stacks.add(1);
    cmd::function(apply_strength_buff);
}

fn main() {
    let commands = on_ate_chorus_fruit();
    assert!(
        commands
            .iter()
            .any(|c| c.contains("scoreboard players add")),
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
                let guard_cmds =
                    sand_core::execute_when::unless(condition).then_one("return 0");
                assert!(
                    guard_cmds
                        .iter()
                        .any(|c| c.contains("score @s") && c.contains("matches ..4")),
                    "guard should enforce str_stacks < 5 (matches ..4); got: {guard_cmds:?}"
                );
                found_guard = true;
            }
        }
    }
    assert!(
        found_guard,
        "on_ate_chorus_fruit event descriptor not found"
    );
}
