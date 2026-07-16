# Events

Sand has two event-definition families. Choose them by how Minecraft detects the
event, not by the handler's name.

| Family | Use it for | Handler parameter |
|---|---|---|
| `AdvancementEvent` | One vanilla advancement trigger | `Event<T>` |
| `SandEvent` | Typed tick observation, owned lifecycle, generic definitions, and same-cycle chaining | A concrete unit marker |

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

## Same-cycle chaining available today

A child `SandEvent` can dispatch from a tick-backed parent's successful cycle:

```rust
use sand_core::condition::Condition;
use sand_core::events::{SandEvent, SandEventDispatch};
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
            .when(Condition::raw("block ~ ~-1 ~ minecraft:white_wool"))
            .into()
    }
}

#[event]
pub fn on_elevator_jump(_event: JumpedOnElevator) {
    cmd::say("Elevator jump");
}
```

The child reuses the parent's detector and inherits the same player `@s`,
position, and tick. A parent does not need its own direct handler. Chains may
nest, one parent may have several children, cycles are rejected, and
multi-plan conditions are coalesced per player. Child observation lifecycle
runs around each child condition attempt.

This is the implemented same-cycle phase, not general event correlation.
Current limits are tracked as `LIM-EXP-004`:

- one direct parent per child;
- tick-backed structured or compatibility-condition parents only;
- no persistent held-state `while_<E>()`;
- no multi-parent `after_any` or `after_all`;
- no bounded `.within(...)` window or cross-tick correlation;
- no advancement-backed graph parents;
- no participant-rich contexts or arbitrary non-player scopes.

Those names describe planned roadmap phases, not callable APIs.

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
