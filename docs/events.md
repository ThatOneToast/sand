# Events

Events connect Rust functions to Minecraft gameplay triggers. Custom
advancement-backed events use `Event<T>` as the handler context, with `T`
implementing `AdvancementEvent`.

```rust
use sand_core::event::trigger::ConsumeItemTrigger;
use sand_core::events::OnJoinEvent;
use sand_core::prelude::*;
use sand_components::ItemPredicate;
use sand_macros::{event, function};

static MANA: ScoreVar<i32> = ScoreVar::new("mana");

pub struct AteGoldenAppleEvent;

impl AdvancementEvent for AteGoldenAppleEvent {
    type Trigger = ConsumeItemTrigger;

    fn trigger() -> Self::Trigger {
        ConsumeItemTrigger::new()
            .item(ItemPredicate::id("minecraft:golden_apple"))
    }

    fn guard() -> Option<Condition> {
        Some(MANA.of("@s").lt(100))
    }
}

#[event]
pub fn on_ate_golden_apple(event: Event<AteGoldenAppleEvent>) {
    MANA.add(event.player(), 10);
    cmd::call(golden_apple_reward);
}

#[function]
pub fn golden_apple_reward() {
    cmd::tellraw(Selector::self_(), Text::new("+10 mana").gold());
}
```

Use `dispatch = "advancement"` only for compatibility with older unit-style
custom event handlers. New custom advancement events should not need it.

Built-in tick/synthetic events can still use unit-style parameters while they
remain on the legacy dispatch path:

```rust
#[event]
pub fn on_join(event: OnJoinEvent) {
    cmd::tellraw(event.player(), Text::new("Welcome").green());
}
```

## Damage Events

Use `DamageEvent<T>` when `T: DamageAdvancementEvent` and the handler needs
damage-specific helpers:

```rust
pub struct EnhancedCellsDamagedEvent;

impl AdvancementEvent for EnhancedCellsDamagedEvent {
    type Trigger = AdvancementTrigger;

    fn trigger() -> Self::Trigger {
        AdvancementTrigger::EntityHurtPlayer {
            entity: None,
            damage: None,
        }
    }
}

impl DamageAdvancementEvent for EnhancedCellsDamagedEvent {}

#[event]
pub fn on_damaged(event: DamageEvent<EnhancedCellsDamagedEvent>) {
    event
        .reflect_damage()
        .to(EntityTargets::nearby(5.0).excluding_players().excluding_self())
        .amount(DamageAmount::fixed(4.0))
        .damage_type(DamageKind::Generic)
        .run();
}
```

`Event<T>` still works for ordinary advancement handlers. `DamageEvent<T>` is
restricted to damage-capable events, so damage-only helpers are not available in
non-damage contexts.

Vanilla advancement rewards do not expose exact damage amount. Reflected damage
uses explicit fixed amounts unless a future tracker is enabled.
