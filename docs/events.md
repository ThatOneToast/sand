# Events

Sand has two event-definition families. Choose them by how Minecraft detects the
event, not by the handler's name.

| Family | Use it for | Handler parameter |
|---|---|---|
| `AdvancementEvent` | One vanilla advancement trigger | `Event<T>` |
| `SandEvent` | Typed tick observation, lifecycle, generic definitions, same-cycle composition, and explicit persistent conditions | A concrete unit marker |

## AdvancementEvent: one stateless vanilla trigger

An `AdvancementEvent` type defines one trigger plus optional reset, guard, ID,
visibility, and state declarations. It is a stateless definition: Sand never
constructs `T`.

```rust
use sand_core::prelude::*;
use sand_macros::event;

pub struct AteGoldenApple;

impl AdvancementEvent for AteGoldenApple {
    type Trigger = ConsumeItemTrigger;

    fn trigger() -> Self::Trigger {
        ConsumeItemTrigger::new()
            .item(ItemPredicate::id("minecraft:golden_apple"))
    }
}

#[event]
pub fn on_ate(event: Event<AteGoldenApple>) {
    cmd::tellraw(event.player(), Text::new("Golden apple eaten").gold());
}
```

`Event<AteGoldenApple>` is the generated runtime context. It currently exposes
the triggering player/subject and common context helpers; it does not contain an
`AteGoldenApple` value. Declaring ordinary Rust fields on the marker does not
make those fields event-time data. Read runtime state through typed Sand state
or through context handles explicitly documented for that event family.

Use `DamageEvent<T>` only when `T: DamageAdvancementEvent` and the handler
needs damage-specific helpers. Reset behavior belongs to
`AdvancementEvent::reset()`; it is not configured with an event attribute.

## SandEvent: advanced custom dispatch

A `SandEvent` owns a custom dispatch plan. `SandEventDispatch::tick()` uses
Sand's typed `Condition` IR, and `SandEvent::setup()` owns objectives plus
commands that run before and after observation.

```rust
use sand_core::events::{EventSetup, SandEvent, SandEventDispatch};
use sand_core::prelude::*;
use sand_macros::event;

static JUMPS: ScoreVar<i32> = ScoreVar::new("jumps");
static PREVIOUS_JUMPS: ScoreVar<i32> = ScoreVar::new("previous_jumps");

pub struct PlayerJumped;

impl SandEvent for PlayerJumped {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tick()
            .as_players()
            .when(PREVIOUS_JUMPS.of("@s").lt_score(JUMPS.of("@s")))
            .into()
    }

    fn setup() -> EventSetup {
        EventSetup {
            objectives: vec![
                "scoreboard objectives add jumps minecraft.custom:minecraft.jump".into(),
                "scoreboard objectives add previous_jumps dummy".into(),
            ],
            pre_observation: vec![],
            post_observation: vec![
                "execute as @a run scoreboard players operation @s previous_jumps = @s jumps"
                    .into(),
            ],
        }
    }
}

#[event]
pub fn on_jump(_event: PlayerJumped) {
    cmd::say("Jumped!");
}
```

The handler parameter is the concrete `SandEvent` marker, not `Event<T>`.
Subscribed markers must therefore be constructible as unit values. Conditions
may use `.when(...)`, `.unless(...)`, or `.if_(...)`; use
`Condition::raw(...)` only as the explicit escape hatch when a typed condition
does not exist.

Several handlers for the same concrete event share one detector and one setup.
Sand sorts generated fan-out, and conflicting definitions for the same event
identity fail export instead of silently choosing one.

### Generic SandEvent definitions

Generic definitions are supported. Each concrete monomorphization has distinct
in-process and generated-resource identity. If the generic type stores
`PhantomData` or other fields, subscribe through a concrete unit adapter:

```rust
use sand_core::condition::Condition;
use sand_core::events::{SandEvent, SandEventDispatch};
use sand_core::prelude::*;
use sand_macros::event;
use std::marker::PhantomData;

pub trait Direction {
    const TAG: &'static str;
}
pub struct Up;
impl Direction for Up {
    const TAG: &'static str = "up";
}

pub struct ElevatorUsed<D: Direction>(PhantomData<D>);

impl<D: Direction> SandEvent for ElevatorUsed<D> {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw(format!("entity @s[tag=elevator_{}]", D::TAG)))
            .into()
    }
}

pub struct ElevatorGoingUp;
impl SandEvent for ElevatorGoingUp {
    fn dispatch() -> SandEventDispatch {
        ElevatorUsed::<Up>::dispatch()
    }
}

#[event]
pub fn on_elevator_up(_event: ElevatorGoingUp) {
    cmd::say("Going up");
}
```

The adapter preserves the generic definition's dispatch while giving generated
handler code a constructible unit marker. Runtime values do not come from the
generic marker's Rust fields.

## Same-cycle, persistent, and bounded composition available today

A child `SandEvent` can dispatch from a tick-backed parent's successful cycle:

```rust
use sand_core::condition::Condition;
use sand_core::events::{PlayerSneakEvent, SandEvent, SandEventDispatch};
use sand_core::prelude::*;
use sand_macros::event;

pub struct PlayerJumped;

impl SandEvent for PlayerJumped {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("entity @s[tag=jumped_this_tick]"))
            .into()
    }
}

pub struct JumpedOnElevator;

impl SandEvent for JumpedOnElevator {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::chain::<PlayerJumped>()
            .while_::<PlayerSneakEvent>()
            .when(Condition::raw("block ~ ~-1 ~ minecraft:white_wool"))
            .into()
    }
}

#[event]
pub fn on_elevator_jump(_event: JumpedOnElevator) {
    cmd::say("Elevator jump");
}

pub struct JumpedOrUsedElevator;

impl SandEvent for JumpedOrUsedElevator {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::after_any::<(PlayerJumped, ElevatorGoingUp)>()
            .while_::<PlayerSneakEvent>()
            .unless(Condition::entity("@s[tag=blocked]"))
            .into()
    }
}
```

Same-cycle occurrence clauses have distinct meanings:

- `chain::<A>()` and `compose().after::<A>()` require `A` to have fired for
  the inherited subject during the current event cycle. `chain::<A>()` remains
  the concise single-parent spelling.
- `after_any::<(A, B)>()` requires at least one listed parent. If several fire
  in the cycle, the child is coalesced to at most one dispatch for that
  subject.
- `after_all::<(A, B)>()` requires every distinct listed parent. Repeating one
  parent cannot substitute for another.

`after_any` and `after_all` accept typed tuples of two through eight concrete
`SandEvent` types. A definition may contain at most one group of each kind;
duplicate parents and repeated groups are rejected. All declared occurrence
clauses are conjunctive, so
`chain::<A>().after_any::<(B, C)>().after_all::<(D, E)>()` means
`A AND (B OR C) AND D AND E`. Additional `.when(...)`, `.unless(...)`, and
`while_::<State>()` requirements are also conjunctive.

Occurrence parents reuse their detectors, including parents referenced only by
a composition. When a graph contains a multi-parent clause, Sand resets the
needed per-player occurrence marks, runs root checks in canonical deterministic
order, and then evaluates composed children in deterministic topological order.
Marks are set before dependent evaluation and do not survive the cycle. The
generated scoreboard reads, writes, resets, and coalescing guards operate as
the inherited player `@s`; one player's occurrence cannot satisfy another
player's child. Registration and tuple order do not affect the generated
ordering.

`while_::<PlayerSneakEvent>()` is different from every `after` form: it queries
whether that player is currently sneaking when the child is evaluated.
`PlayerSneakEvent` does not need to fire, and its detector or lifecycle is not
invoked. The current condition is usable on the first observation; there is no
transition-baseline suppression.

Persistent state is explicit. A type must implement `PersistentSandEvent`;
ordinary tick events, transitions such as `PlayerStartSneakingEvent`, and
advancement events do not become persistent merely because they are events.
Custom persistent providers must return an independently valid condition and
an empty `SandEvent::setup()`; shared objectives or other prerequisites belong
in typed state lifecycle. Export rejects a provider setup instead of silently
omitting it.
The built-in persistent states currently include sneaking, sprinting, swimming,
flying, on-fire, and Creative/Adventure/Spectator mode. Multiple `while_`
requirements are ANDed and compose with `.when(...)` and `.unless(...)`.

`within::<E>(TickWindow::new(N)?)` is bounded cross-tick correlation: `E` must
have fired for the inherited subject during the current cycle **or** within
the previous `N - 1` completed tick boundaries.

```rust
use sand_core::events::{SandEvent, SandEventDispatch, TickWindow};
use sand_core::prelude::*;
use sand_macros::event;

pub struct SwitchPulled;
impl SandEvent for SwitchPulled {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tick().as_players().into()
    }
}

pub struct EnteredVault;
impl SandEvent for EnteredVault {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tick().as_players().into()
    }
}

pub struct VaultOpenedAfterSwitch;
impl SandEvent for VaultOpenedAfterSwitch {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::compose()
            .after::<EnteredVault>()
            .within::<SwitchPulled>(TickWindow::new(20).expect("nonzero, in range"))
            .into()
    }
}

#[event]
pub fn on_vault_opened(_event: VaultOpenedAfterSwitch) {
    cmd::say("Vault opened within 20 ticks of the switch");
}
```

Internally, Sand tracks one exact per-subject age (ticks elapsed since `E`
last fired, reset to `0` the cycle `E` fires) shared by every child and window
referencing that parent — each child's resolved condition just compares that
one age against its own `N - 1`, so distinct windows on the same parent never
need distinct or lossy state. `N = 1` is therefore identical to
`after::<E>()`: only a same-cycle occurrence satisfies it. A parent firing on
the current tick always satisfies `within` regardless of its prior age — the
age is refreshed to `0` before any staged child reads it, using the same
per-subject occurrence mark that same-cycle composition already establishes.
A later parent occurrence always refreshes the window; a bounded parent
occurrence never directly dispatches the child, and repeated parent
occurrences refresh state rather than queueing deliveries. `TickWindow`
rejects `0` and windows above 24,000 ticks (20 minutes) — it is a bounded
correlation window, not a session/persistence mechanism. The age update runs
under `execute as @a`, which only reaches online players, so **age advances
only while a player is online and pauses while they are offline**; the score
itself is not reset by disconnect/reconnect or `/reload` (it persists like
existing `Cooldown`/`Timer` scoreboard state), so a returning player resumes
aging from wherever it stopped rather than restarting it. The increment is
also guarded to stop at `TickWindow::MAX_TICKS` (24,000): Minecraft
scoreboard values are signed 32-bit, and an unguarded increment on a
permanently-idle parent would eventually overflow and wrap negative, which
would incorrectly re-satisfy `age <= N - 1` for every window; since
`TickWindow::MAX_TICKS` is the largest representable window, an age that has
reached it is already permanently expired for every valid window. Distinct
concrete parent types compose conjunctively with `within`, same as
`after`/`after_any`/`after_all`; a repeated `.within` call for the same
parent and window is deduplicated, and a conflicting window for the same
parent is rejected at export.

Evaluation remains per player with inherited `@s` and position. For the
single-parent path, the live condition is tested after that parent's direct
handlers and before its post-observation lifecycle. Multi-parent children are
tested after their required root occurrence marks have been established and
before parent post-observation lifecycle advances tracked state. A
parent does not need its own direct handler. Compositions may nest, one parent
may have several children or groups, mixed dependency cycles are rejected with
labeled paths, and multi-plan conditions are coalesced per player. Child
observation lifecycle runs around each child condition attempt;
post-observation is deferred through downstream staged dependents, including
mixed graphs with an immediate single-parent intermediate.

This is the implemented same-cycle, persistent, and bounded-correlation
composition surface, not general event correlation. Current limits are
tracked as `LIM-EXP-004`:

- tick-backed structured or compatibility-condition parents only;
- bounded correlation is capped at 24,000 ticks (`TickWindow::MAX_TICKS`) —
  not an unbounded historical event log or session mechanism;
- no advancement-backed graph parents;
- no participant-rich contexts or arbitrary non-player scopes.

## Built-in events

Built-in advancement-backed and generated tracked events use a special
name-dispatched macro path with the same `Event<T>` runtime context. They do
not necessarily implement `AdvancementEvent`, so only use the context methods
documented for that built-in type:

```rust
use sand_core::events::OnJoinEvent;
use sand_core::prelude::*;
use sand_macros::event;

#[event]
pub fn on_join(event: Event<OnJoinEvent>) {
    cmd::tellraw(event.player(), Text::new("Welcome").green());
}
```

Continuous tick-only built-ins such as `PlayerSneakEvent` implement
`SandEvent` and use their concrete unit marker as the handler parameter.

See [Advancement Events](advancement-events.md) for the lightweight family and
the [mdBook event chapter](../book/src/manual/events.md) for the same canonical
model in the user guide.
