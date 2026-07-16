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
                "execute as @a run scoreboard players operation @s sync_jumps = @s jumps".into(),
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
generated dispatch function (handler paths sorted, so output doesn't depend on
registration order) — not one detector per handler. If two handlers claim to
subscribe to the same event type but their `dispatch()`/`setup()` results
actually differ, export fails with a diagnostic rather than silently picking
one definition.

Generic `SandEvent` families (e.g. `ElevatorUsed<GoUp>` vs `ElevatorUsed<GoDown>`)
each get a distinct identity per concrete monomorphization. Two separate
identity concepts are involved, deliberately kept apart:

- `TypeId` distinguishes concrete event types — including distinct generic
  monomorphizations — **during one export process**. It is used only for
  in-process grouping/deduplication and is *not* a stable identifier across
  compiler versions or builds.
- Generated datapack resource paths (the detector, setup, and dispatch
  function names) use a deterministic key derived from the canonical concrete
  event type identity (`std::any::type_name::<T>()`), not from `TypeId` and
  not from the set of subscribed handler paths — so adding, removing, or
  reordering handlers never renames an event's generated resources.

Conditions that expand into more than one OR-alternative execute plan (e.g. a
top-level `Any`) emit one detection line per plan, guarded so at most one
dispatch happens per player per tick even if more than one plan matches on the
same tick.

## Composing SandEvents: same-cycle chained dispatch

`SandEventDispatch::tick()` detects an event independently — its own
condition, polled every tick. `SandEventDispatch::chain::<Parent>()` instead
declares that an event fires **only from `Parent`'s successful dispatch
cycle**: same execution subject (`@s`), same position, same tick. The child
reuses `Parent`'s detector rather than independently re-polling it.

```text
SandEvent::tick()
- detects an event independently, polled every tick.

SandEvent::chain::<Parent>()
- evaluates only when Parent's detector fires this cycle.
- reuses Parent's detector — no duplicate polling.
- inherits Parent's player execution context (@s, position).
```

```rust,ignore
use sand_core::condition::Condition;
use sand_core::events::{SandEvent, SandEventDispatch};

pub struct JumpedOnElevator;

impl SandEvent for JumpedOnElevator {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::chain::<PlayerJumpEvent>()
            .when(Condition::raw("block ~ ~-1 ~ minecraft:white_wool"))
            .into()
    }
}
```

`Parent` (here `PlayerJumpEvent`) does **not** need a direct `#[event]`
handler of its own — Sand discovers it recursively from the chain reference
and still generates its detector/setup. `.when(...)`/`.unless(...)`/`.if_(...)`
work exactly as they do on `SandEventDispatch::tick()`, built from the same
typed `Condition` IR.

Generated output for the example above: the parent's dispatch function fans
out to its own direct handlers first, then to each child edge, in
deterministic order (handler paths sorted, then child edges sorted by
canonical child type name):

```mcfunction
# __sand_event_dispatch/<parent key>
function mypack:on_jump
execute if block ~ ~-1 ~ minecraft:white_wool run function mypack:on_jumped_on_elevator
```

No `execute as @a` is re-issued for the child — it inherits the current `@s`
and position from the parent's own detection line. A child with no
conditions is called unconditionally (no `execute if`/`unless` wrapper); a
child whose conditions expand into more than one OR-alternative plan gets a
per-player coalescing guard (mirroring the guard used for a root's own
multi-plan detection) so it fires at most once per parent invocation, even if
more than one of its plans matches the same tick.

Chains can nest to arbitrary depth (`A -> B -> C`) and one parent can have
several children — including several concrete instantiations of a generic
event family (`ElevatorUsed<GoUp>` / `ElevatorUsed<GoDown>`), each keeping its
own distinct identity, condition, and generated dispatch resource, while
sharing the same parent detector.

A parent's `pre_observation`/`post_observation` still run every tick,
unconditionally, around detection — child dispatch (and everything it
reaches) always completes before the parent's own `post_observation`, so a
child relying on a delta-tracking parent condition (e.g. the jump-count sync)
never observes already-synchronized state.

### Child lifecycle ordering

A chained child can own its own `EventSetup` lifecycle, exactly like a root
`SandEvent`. For each successful parent occurrence, a child performs
pre-observation, evaluates its chain conditions, optionally dispatches its
handlers and descendants, and then performs post-observation:

```text
child pre_observation
child condition evaluation
child handler and descendant dispatch, if matched
child post_observation
```

Post-observation runs after **every** child observation attempt, not only a
successful child dispatch — the same "advance synchronized state every
cycle" contract that already applies to a root's own `pre_observation`/
`post_observation`. This matters whenever a child's own condition depends on
state its `pre_observation` prepares, or whenever its `post_observation`
must advance a delta-tracking score regardless of whether the condition
matched this cycle:

```rust,ignore
pub struct Child;

impl SandEvent for Child {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<Parent>()
            .when(SYNC.of("@s").lt_score(CURRENT.of("@s")))
    }

    fn setup() -> EventSetup {
        EventSetup {
            pre_observation: vec![
                "scoreboard players operation @s current = @s source".into(),
            ],
            post_observation: vec![
                "scoreboard players operation @s sync = @s current".into(),
            ],
            ..EventSetup::none()
        }
    }
}
```

A child with lifecycle commands gets a dedicated `__sand_event_observe/<child>`
function wrapping its condition test between `pre_observation` and
`post_observation`; the parent calls it unconditionally, and the condition
test (single-plan, multi-plan-guarded, or unconditional) lives inside it:

```mcfunction
# __sand_event_observe/<child>
scoreboard players operation @s current = @s source
execute if score @s sync < @s current run function pack:__sand_event_dispatch/<child>
scoreboard players operation @s sync = @s current
```

Do not assume lifecycle setup commands execute only after the child
condition succeeds — `post_observation` is a standalone command in the
observe function, not embedded inside the `execute ... run function` line,
so it is structurally reached whether or not the condition holds at
runtime. A child with no lifecycle commands keeps the simpler direct-call
shape (no observe function) shown earlier in this section. This ordering
applies at every chain depth: each chained node's lifecycle is tied to each
invocation of its own parent, not to the global tick independently.

### Limitations in this phase

This is the first, same-cycle-only phase of chained dispatch (#240). Not yet
implemented, and tracked as future phases of the same issue:

- `while_<E>()` (continuous/held-state events)
- `after_all(...)` / `after_any(...)` (multi-parent joins)
- bounded `.within(...)` time windows
- cross-tick correlation
- participant-rich execution contexts (#230)
- arbitrary (non-player) entity execution scopes

Each event may currently have **at most one** direct parent. A dependency
cycle (`A -> A`, or an indirect cycle like `A -> B -> C -> A`) is rejected at
export time with a diagnostic naming the full cycle path — never silently
truncated or panicked on.

Structured `SandEventDispatch::tick()` and legacy `SandEventDispatch::TickCondition`
parents are supported — both normalize into the same tick-lifecycle shape and
are discovered identically, so a parent resolves to exactly one generated
detector regardless of which constructor it used. Advancement-backed parents
are rejected in this phase: `AdvancementTrigger` dispatch normalizes into a
separate `Advancement` shape, and its reward-function generation path does
not yet provide a player execution context compatible with same-cycle child
dispatch:

```text
SandEvent `JumpedOnElevator` cannot chain from `SomeAdvancementBackedEvent`:
parent dispatch scope does not provide a player execution context
(advancement-backed SandEvent parents are not yet supported by chained
dispatch — see #240)
```

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
| `ItemObtainedTrigger` | Removed `minecraft:crafted_item` (compatibility only; current export rejects it) | `item(predicate)` |
| `ItemEnchantTrigger` | `minecraft:enchanted_item` | `item(predicate)`, `levels(predicate)` |
| `RecipeUnlockedTrigger` | `minecraft:recipe_unlocked` | `new(recipe)` |
| `MultiKillTrigger` | `minecraft:killed_by_arrow` | `unique_entity_types(n)`, `victim(predicate)` |
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
