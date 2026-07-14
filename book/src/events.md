# Events

Events connect Rust functions to Minecraft gameplay triggers. Annotate a function
with `#[event]` and Sand generates the advancement JSON + reward function wire-up
at build time.

## The canonical split: `AdvancementEvent` vs `SandEvent`

Sand has two event abstractions, chosen for different jobs:

```text
AdvancementEvent
- one vanilla advancement trigger
- stateless marker — Sand never constructs an instance of T
- handled through Event<T>
- no marker fields as runtime values

SandEvent
- advanced custom event
- typed tick polling and custom dispatch (SandEventDispatch::tick())
- lifecycle-owned setup and observation (SandEvent::setup())
- generic event families with distinct, stable per-monomorphization identity
- future composition and richer contexts (#240 / #230)
```

Use `AdvancementEvent` when your event maps to exactly one vanilla advancement
trigger — it's the lightweight, common case (see the sections below). Reach for
`SandEvent` when you need a typed tick condition, owned lifecycle resources
(objectives, pre/post-observation commands), or a generic event family.

### Typed `SandEvent` tick dispatch

`SandEventDispatch::tick()` builds a structured, typed condition instead of a
hand-formatted string, reusing the same `Condition`/`ScoreVar` IR used
everywhere else in Sand:

```rust,ignore
use sand_core::events::{EventSetup, SandEvent, SandEventDispatch};
use sand_core::prelude::*;

static JUMPS: ScoreVar<i32> = ScoreVar::new("jumps");
static SYNC_JUMPS: ScoreVar<i32> = ScoreVar::new("sync_jumps");

pub struct PlayerJumpEvent;

impl SandEvent for PlayerJumpEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tick()
            .as_players()
            .when(SYNC_JUMPS.of("@s").lt_score(JUMPS.of("@s")))
            .into()
    }

    fn setup() -> EventSetup {
        EventSetup {
            objectives: vec![
                "scoreboard objectives add jumps minecraft.custom:minecraft.jump".into(),
                "scoreboard objectives add sync_jumps dummy".into(),
            ],
            pre_observation: vec![],
            // Runs unconditionally after detection, every tick — so the
            // synchronized score never gets ahead of the value being
            // compared against (detect-then-sync ordering).
            post_observation: vec![
                "scoreboard players operation @a sync_jumps = @a jumps".into(),
            ],
        }
    }
}
```

`.when(...)`/`.unless(...)` accept anything that converts into a `Condition` —
score comparisons, flags, predicates, entity checks — or the explicit escape
hatch `Condition::raw("...")` for fragments with no typed equivalent yet.
`.if_(...)` is an alias for `.when(...)`.

When several `#[event]` handlers subscribe to the same `SandEvent` type, Sand
deduplicates the detector: one shared generated tick function and one copy of
`setup()`'s objectives, with all handler bodies fanned out from a single
generated dispatch function — not one detector per handler.

Generic `SandEvent` families (e.g. `ElevatorUsed<GoUp>` vs `ElevatorUsed<GoDown>`)
are grouped by `TypeId` only within the exporting Rust process, so concrete
instantiations never merge detectors. Generated resource paths instead use a
deterministic hash of the event type's canonical macro spelling; adding,
removing, or reordering handlers does not rename them.

## Tracked transitions

Start/stop sneaking is the proof event pair for Sand's reusable transition
backend. Both handlers share one private tracker:

```rust,ignore
use sand_core::event::vanilla::{PlayerStartsSneaking, PlayerStopsSneaking};
use sand_core::prelude::*;
use sand_macros::event;

#[event]
fn sneak_started(event: Event<PlayerStartsSneaking>) {
    cmd::tellraw(event.player(), Text::new("Sneaking started"));
}

#[event]
fn sneak_stopped(event: Event<PlayerStopsSneaking>) {
    cmd::tellraw(event.player(), Text::new("Sneaking stopped"));
}
```

Sand samples vanilla's entity-predicate `flags.is_sneaking` value once per
online player per tick. That entity-predicate flag is available throughout
Sand's supported Java Edition target range. The first sample stores a baseline
and fires nothing. Every handler for an edge runs before the shared previous
value is updated, so handler order cannot suppress another handler.

This is tick-polled reliability, not an exact key-input event. A reload keeps
the scoreboard baseline and does not itself fire. Offline players are not
sampled. On rejoin, retained tracker state is compared with the first new
sample; if sneaking differs from the last observed online state, a transition
can fire on that tick. Vanilla datapacks cannot observe state changes while a
player is offline.

Runtime cost for this proof tracker is three private scoreboard objectives, one
predicate sample per online player per tick, transition comparisons for the
registered edge kinds, and three baseline updates. Identical declarations and
multiple handlers share that cost. Use the continuous `PlayerSneakEvent` for
every-tick behavior, or raw/manual commands when custom sampling semantics are
needed.

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
| `#[event]` for `PlayerStartsSneaking` / `PlayerStopsSneaking` | Shared predicate/score transition tracker |
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
| `ItemObtainedTrigger` (crafted items only) | `minecraft:crafted_item` | `item(predicate)` |
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
- Custom tick-polled or lifecycle-owned events: implement `SandEvent` (see
  [The canonical split](#the-canonical-split-advancementevent-vs-sandevent) above)

`Event<T>` is the typed handler context; use `event.player()` for the player selector. `AdvancementEvent` is the architecture for typed advancement triggers, while custom-item extension events and `Interactable` generate the same advancement-plus-reward-function shape. Damage events add source-aware advancement criteria; `UsingItemTrigger` and `SummonedEntityTrigger` cover their vanilla trigger semantics.

<div class="sand-warning"><strong>Vanilla event boundary.</strong> Sand cannot invent a Minecraft trigger. Some actions have no advancement trigger and must be modeled with a tick system, scoreboards, inventory checks, or damage tracking. Use a guard to reject unwanted advancement matches rather than assuming all gameplay context is present.</div>
