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

## Same-cycle and persistent composition available today

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

This is the implemented same-cycle phase, not general event correlation.
Current limits are tracked as `LIM-EXP-004`:

- tick-backed structured or compatibility-condition parents only;
- no bounded `.within(...)` window or cross-tick correlation;
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
