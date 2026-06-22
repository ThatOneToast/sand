# Events

Events connect Rust functions to Minecraft gameplay triggers. Annotate a function
with `#[event]` and Sand generates the advancement JSON + reward function wire-up
at build time.

```rust
use sand_core::prelude::*;
use sand_macros::event;

static MANA: ScoreVar<i32> = ScoreVar::new("mana");

#[event(dispatch = "advancement")]
pub fn on_eat_golden_apple(event: ItemConsumeEvent) {
    MANA.add(event.player(), 10);
    cmd::tellraw(event.player(), Text::new("+10 mana!").green());
}
```

## Dispatch modes

| Mode | How it fires |
|---|---|
| `#[event]` for `OnJoinEvent` | Tick tag check — every session join |
| `#[event]` for `FirstJoinEvent` | Tick advancement (no revoke) — once per player ever |
| `#[event]` for `OnDeathEvent` / `OnRespawnEvent` | Death count scoreboard — tick-based |
| `#[event(dispatch = "advancement")]` for custom types | `AdvancementEvent::trigger()` type + `AdvancementEvent::guard()` |
| `#[event]` for `HoldingItemEvent` / `CurrentlyWearingEvent` | Per-tick `execute if items` |

## Custom events with `AdvancementEvent`

Define a marker struct and implement `AdvancementEvent` + `EventPlayer`:

```rust
use sand_core::event::AdvancementEvent;
use sand_core::event::trigger::ConsumeItemTrigger;

pub struct AteGoldenAppleEvent;

impl AdvancementEvent for AteGoldenAppleEvent {
    type Trigger = ConsumeItemTrigger;
    fn trigger() -> Self::Trigger {
        ConsumeItemTrigger::new()
            .item(serde_json::json!({"items": "minecraft:golden_apple"}))
    }
    fn guard() -> Option<Condition> {
        Some(MANA.of("@s").lt(100))
    }
}

impl EventPlayer for AteGoldenAppleEvent {
    fn player(&self) -> Selector { Selector::self_() }
}
```

## Typed trigger builders

Available in `sand_core::event::trigger`:

| Builder | Minecraft trigger | Methods |
|---|---|---|
| `TickTrigger` | `minecraft:tick` | — |
| `ImpossibleTrigger` | `minecraft:impossible` | — |
| `ConsumeItemTrigger` | `minecraft:consume_item` | `item(predicate)` |
| `UsingItemTrigger` | `minecraft:using_item` | `item(predicate)` |
| `PlayerKilledEntityTrigger` | `minecraft:player_killed_entity` | `entity(predicate)`, `killing_blow(predicate)` |
| `EntityKilledPlayerTrigger` | `minecraft:entity_killed_player` | `entity(predicate)`, `killing_blow(predicate)` |
| `InventoryChangedTrigger` | `minecraft:inventory_changed` | `slots(predicate)`, `item(predicate)` |
| `ItemObtainedTrigger` | `minecraft:recipe_crafted` | `item(predicate)` |
| `ItemEnchantTrigger` | `minecraft:enchanted_item` | `item(predicate)`, `levels(predicate)` |
| `RecipeUnlockedTrigger` | `minecraft:recipe_unlocked` | `new(recipe)` |
| `MultiKillTrigger` | `minecraft:killed_by_crossbow` | `unique_entity_types(n)`, `victim(predicate)` |

## Guard conditions

Return a `Condition` from `guard()` to prevent the event from firing unless
additional requirements are met. Sand prepends `execute unless <condition> run return 0`
to the reward function:

```rust
fn guard() -> Option<Condition> {
    Some(all![
        MANA.of("@s").gte(25),
        DASH.ready("@s"),
        SHIELD.of("@s").is_false(),
    ])
}
```

## Event handle

`EventHandle` lets you enable, disable, or reset an event per player using
scoreboard objectives (`__ev_<hash>`):

```rust
static GOLDEN_APPLE_HANDLE: EventHandle = EventHandle::new("my_pack:on_ate_golden_apple");

// In load function:
GOLDEN_APPLE_HANDLE.define();

// In death handler:
GOLDEN_APPLE_HANDLE.disable("@s");

// In respawn handler:
GOLDEN_APPLE_HANDLE.enable("@s");
```

## Function pointer calls

Use `cmd::call(fn_ptr)` to call a `#[function]` by pointer instead of string path:

```rust
#[function]
pub fn reward_effect() {
    cmd::say("Reward triggered!");
}

#[event(dispatch = "advancement")]
pub fn on_event(event: SomeEvent) {
    cmd::call(reward_effect as fn() -> Vec<String>);
}
```

## Built-in events

Sand ships 50+ event types in `sand_core::events`. The most common:

- `OnJoinEvent` / `FirstJoinEvent` — player joins
- `OnDeathEvent` / `OnRespawnEvent` — death & respawn
- `ArmorEquipEvent` / `ArmorUnequipEvent` — equipment changes
- `HoldingItemEvent` / `CurrentlyWearingEvent` — per-tick item checks
- `ItemConsumeEvent` — eating/drinking
- Custom: implement `SandEvent` or `AdvancementEvent` for your own types
