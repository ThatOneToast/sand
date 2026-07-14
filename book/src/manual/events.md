# Events

Sand has two event abstractions: `AdvancementEvent` — a stateless marker for one
vanilla advancement trigger, handled through `Event<T>` — and `SandEvent`, the
primary extension mechanism for advanced custom events with typed tick dispatch
and owned lifecycle (setup objectives, pre/post-observation commands). See
[Events](../events.md#the-canonical-split-advancementevent-vs-sandevent) for the
full split and a typed tick-dispatch example.

This page focuses on the common `AdvancementEvent` case: a typed trigger becomes
advancement JSON, and its reward points to a generated function. The handler
receives `Event<T>` and `event.player()` is the triggering player (`@s` in the
reward function).

## Minimal example: using an item

```rust
use sand_core::event::trigger::UsingItemTrigger;
use sand_core::prelude::*;
use sand_macros::event;

pub struct UsingWand;
impl AdvancementEvent for UsingWand {
    type Trigger = UsingItemTrigger;
    fn trigger() -> Self::Trigger {
        UsingItemTrigger::new().item(ItemPredicate::id("minecraft:stick"))
    }
}

#[event]
pub fn on_wand_use(event: Event<UsingWand>) {
    cmd::tellraw(event.player(), Text::new("Wand used").aqua());
}
```

## Practical examples

An altar uses `PlayerInteractedWithEntityTrigger` with an `EntityPredicate` tag. A player-hurt event uses `AdvancementTrigger::EntityHurtPlayer { entity, damage }`; use `DamageEvent<T>` only when `T: DamageAdvancementEvent`. A summon event uses `SummonedEntityTrigger::new().entity(...)`. Custom items add the same reward wiring through `item.on_use_fn(location, handler)`.

```rust
let altar_trigger = PlayerInteractedWithEntityTrigger::new()
    .entity(EntityPredicate::new().nbt("{Tags:[\"arcane_altar\"]}"));
let summoned = SummonedEntityTrigger::new().entity(EntityPredicate::new());
```

## Guards and generated output

`fn guard() -> Option<Condition>` adds a generated early guard to the reward function. Sand serializes the typed trigger into an advancement criterion, emits the reward function, and configures reset/revoke behavior from the event configuration. Use a typed function reference for follow-up functions instead of reward strings.

## When to use tick systems

Use an advancement event for a discrete vanilla action. Use tick logic for continuous state, cooldown decrementing, held-item polling, or an action Minecraft does not expose as a trigger.

| Wanted event | Advancement trigger? | Best pattern | Caveat |
|---|---|---|---|
| Using a wand | Often | `UsingItemTrigger` / `CustomItemExt` | exact semantics depend on vanilla trigger |
| Right-click altar | Yes | tagged `Interactable` | interaction entity only |
| Player takes damage | Yes | hurt trigger + `DamageEvent` | no exact numeric reward payload |
| Exact shield block | No dedicated signal | held/use + damage tracker | approximation only |
| Axe shield disable | No dedicated signal | tick/item/damage pattern | no exact event |
| Inventory changed | Yes | inventory trigger | may fire for many changes |
| Arbitrary right-click item | Limited | item trigger or tick pattern | vanilla decides coverage |

## Common mistakes

- Forgetting event output is advancement JSON plus a function, not a live Rust callback.
- Expecting one trigger to expose all action context.
- Using a tick event for an action that already has a precise advancement trigger.

## Related pages

[Advancement Triggers](advancement-triggers.md), [Item Events](item-events.md), [Entities](entities.md), and [Damage Tracking](damage-tracking.md).
