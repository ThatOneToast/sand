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
