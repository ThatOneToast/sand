# Advancement Events

`AdvancementEvent` is the lightweight, stateless event family for one vanilla
advancement trigger. An advancement event type owns these definition hooks:

- `trigger()` returns the typed Minecraft advancement trigger.
- `guard()` returns an optional `Condition`.
- `reset()` controls whether the advancement is revoked after firing.
- `id()` and `visibility()` control generated advancement metadata.
- `state_defines()` declares typed state required by the event.

The handler receives generated runtime context `Event<T>`, not `T`:

```rust
use sand_core::prelude::*;
use sand_macros::event;

pub struct UsedDashWandEvent;

impl AdvancementEvent for UsedDashWandEvent {
    type Trigger = UsingItemTrigger;

    fn trigger() -> Self::Trigger {
        UsingItemTrigger::new().item(ItemPredicate::id("minecraft:stick"))
    }
}

static MANA: ScoreVar<i32> = ScoreVar::new("mana");

#[event]
pub fn on_used_dash_wand(event: Event<UsedDashWandEvent>) {
    MANA.remove(event.player(), 25);
    cmd::say("Dash!");
}
```

The generated advancement path uses `T: AdvancementEvent` directly, so guards
stay typed as `Option<Condition>` and never flow through legacy string
conditions.

Sand does not construct `T`. Ordinary Rust fields declared on the definition
type are not event-time payload and do not appear on `Event<T>`; read runtime
state through documented context handles or typed Sand state.

For typed tick dispatch, lifecycle ownership, generic event definitions, and
same-cycle chaining, use the advanced `SandEvent` family described in
[Events](events.md).
