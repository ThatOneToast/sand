# Events

Events connect Rust functions to Minecraft gameplay triggers. Annotate a function
with `#[event]` and Sand generates the advancement JSON + reward function wire-up
at build time.

## XP level-up (`PlayerLevelUpEvent`)

Sand provides a working `PlayerLevelUpEvent` backed by a generated
scoreboard/tick system. Use it as any other `#[event]` handler:

```rust
use sand_core::event::vanilla::PlayerLevelsUp;  // shorter alias
use sand_core::events::PlayerLevelUpEvent;       // long form
use sand_core::prelude::*;
use sand_macros::event;

static MANA: ScoreVar<i32> = ScoreVar::new("mana");

#[event]
pub fn on_level_up(event: Event<PlayerLevelUpEvent>) {
    MANA.add(event.player(), 10);
}
```

### Why not `minecraft:leveled_up`?

Vanilla Minecraft does **not** have a `minecraft:leveled_up` advancement trigger.
Any datapack that emits one is rejected at load time. Sand models level-up as a
generated scoreboard/tick system instead:

| Generated objective | Contents |
|---|---|
| `__sand_xp_lvl`   | Current XP level, refreshed every tick |
| `__sand_xp_prev`  | XP level from the previous tick |
| `__sand_xp_delta` | `current − previous` (≥ 1 when handler fires) |
| `__sand_xp_seen`  | Join-safety flag; prevents a false fire on first tick |

### Behaviour

- **First tick after join**: the system initialises `__sand_xp_prev` to the
  player's current level. No handler fires.
- **Level increase**: if `__sand_xp_delta >= 1`, all registered handlers fire.
- **Level decrease or no change**: handlers do not fire.
- **Multiple handlers**: all fire from the same generated `__sand_xp_check` tick
  function; only one tick function is added to `minecraft:tick`.

### Helper methods

Use `PlayerLevelUpEvent::current_level`, `previous_level`, and `level_delta` to
build typed conditions inside your handler without knowing the objective names:

```rust
#[event]
pub fn on_level_up(event: Event<PlayerLevelUpEvent>) {
    // Give mana on any level-up.
    MANA.add(event.player(), 10);

    // Bonus for gaining 5+ levels at once (e.g. via XP bottles).
    let big_jump = PlayerLevelUpEvent::level_delta("@s").gte(5);
    when(big_jump).then_one(cmd::tellraw(
        event.player(),
        Text::new("Massive level up!").gold(),
    ));
}
```

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
| `#[event]` for `OnJoinEvent` | Scoreboard-backed tick check — after load/reload or new player |
| `#[event]` for `FirstJoinEvent` | Tick advancement (no revoke) — once per player ever |
| `#[event]` for `OnDeathEvent` / `OnRespawnEvent` | Death count scoreboard — tick-based |
| `#[event]` for `PlayerLevelUpEvent` / `PlayerLevelsUp` | XP scoreboard/tick system — no advancement |
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
