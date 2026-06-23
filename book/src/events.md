# Events

Events connect Rust functions to Minecraft gameplay triggers. Annotate a function
with `#[event]` and Sand generates the advancement JSON + reward function wire-up
at build time.

```rust
use sand_core::prelude::*;
use sand_core::event::trigger::ConsumeItemTrigger;
use sand_components::ItemPredicate;
use sand_macros::event;

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
pub fn on_eat_golden_apple(event: Event<AteGoldenAppleEvent>) {
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
| `#[event]` with `Event<T>` | `T: AdvancementEvent` trigger + typed `Condition` guard |
| `#[event]` for `HoldingItemEvent` / `CurrentlyWearingEvent` | Per-tick `execute if items` |

## Custom events with `AdvancementEvent`

Define a marker struct and implement `AdvancementEvent`:

```rust
use sand_core::event::trigger::ConsumeItemTrigger;
use sand_core::prelude::*;
use sand_components::ItemPredicate;

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
| `PlayerInteractedWithEntityTrigger` | `minecraft:player_interacted_with_entity` | `item(predicate)`, `entity(predicate)` |
| `SummonedEntityTrigger` | `minecraft:summoned_entity` | `entity(predicate)` |

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
static GOLDEN_APPLE_HANDLE: EventHandle<AteGoldenAppleEvent> = EventHandle::new();

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

#[event]
pub fn on_event(event: Event<SomeEvent>) {
    cmd::call(reward_effect);
}
```

## Built-in events

Sand ships 50+ event types in `sand_core::events`. The most common:

- `OnJoinEvent` / `FirstJoinEvent` — player joins
- `OnDeathEvent` / `OnRespawnEvent` — death & respawn
- `ArmorEquipEvent` / `ArmorUnequipEvent` — equipment changes
- `HoldingItemEvent` / `CurrentlyWearingEvent` — per-tick item checks
- `ItemConsumeEvent` — eating/drinking
- Custom advancement events: implement `AdvancementEvent` and handle `Event<T>`
- Legacy/custom tick-poll events: implement `SandEvent`

`Event<T>` is the typed handler context; use `event.player()` for the player selector. `AdvancementEvent` is the architecture for typed advancement triggers, while custom-item extension events and `Interactable` generate the same advancement-plus-reward-function shape. Damage events add source-aware advancement criteria; `UsingItemTrigger` and `SummonedEntityTrigger` cover their vanilla trigger semantics.

<div class="sand-warning"><strong>Vanilla event boundary.</strong> Sand cannot invent a Minecraft trigger. Some actions have no advancement trigger and must be modeled with a tick system, scoreboards, inventory checks, or damage tracking. Use a guard to reject unwanted advancement matches rather than assuming all gameplay context is present.</div>
