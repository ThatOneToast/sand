# Typed State Flows

`StateFlow<S>` connects enum-backed `GameState<S>` values, typed conditions,
guarded transitions, and enter/exit/while-in behavior through the existing
transition and lifecycle exporter.

```rust,ignore
use sand::prelude::*;
use sand::{component, function};

#[derive(Clone, Copy, PartialEq, Eq)]
enum BossPhase { Idle, Fighting, Enraged, Defeated }

impl TypedGameState for BossPhase {
    fn to_score(self) -> i32 { self as i32 }
    fn from_score(score: i32) -> Option<Self> {
        match score {
            0 => Some(Self::Idle),
            1 => Some(Self::Fighting),
            2 => Some(Self::Enraged),
            3 => Some(Self::Defeated),
            _ => None,
        }
    }
}

static PHASE: GameState<BossPhase> =
    GameState::with_default_score("boss_phase", 0);
static HEALTH_PERCENT: ScoreVar<i32> = ScoreVar::new("boss_health_pct");

#[function]
fn start_enrage() { cmd::say("Enraged!"); }

#[function]
fn stop_fighting() { cmd::say("Fight ended"); }

#[function]
fn enraged_tick() {
    Actionbar::show(Selector::self_(), Text::new("ENRAGED").dark_red());
}

#[datapack_component(Load)]
fn boss_flow() {
    StateFlow::players(&PHASE)
        .named("boss")
        .transition(BossPhase::Fighting, BossPhase::Enraged)
            .when(HEALTH_PERCENT.of("@s").lte(50))
            .priority(100)
            .done()
        .transition(BossPhase::Fighting, BossPhase::Defeated)
            .when(HEALTH_PERCENT.of("@s").lte(0))
            .priority(200)
            .done()
        .on_exit(BossPhase::Fighting, cmd::call(stop_fighting))
        .on_enter(BossPhase::Enraged, cmd::call(start_enrage))
        .on_tick(BossPhase::Enraged, cmd::call(enraged_tick))
        .register();
}
```

Registration happens while Sand collects components for export; it does not
run the flow on datapack load. The exporter creates the objectives, private
helpers, and `minecraft:tick` wiring.

## Conflict and ordering contract

For each selected subject and server tick:

1. A missing score receives the `GameState` default, if one exists. This
   initialization does not count as a transition and does not run enter hooks.
2. Transitions are evaluated by descending priority.
3. Equal-priority transitions preserve declaration order.
4. A private per-subject lock means the first successful transition wins;
   lower-priority or later equal-priority transitions cannot write again in
   that cycle.
5. A successful `A → B` runs exit hooks for `A`, writes `B`, then runs enter
   hooks for `B`.
6. Tick hooks are evaluated after transitions, so a newly entered state may
   run its while-in hook in the same server tick.

No hook runs for `A → A`; identical-source/destination transitions are
diagnosed. Hooks do not repeat while the state remains unchanged. Resetting to
a typed default through a registered transition has normal exit/write/enter
semantics. Direct `GameStateRef::set`, `reset`, or raw scoreboard writes are
low-level escape hatches and intentionally bypass flow hooks. Clearing a state
is not a flow target in this API.

For a compact one-off write without registration:

```rust,ignore
PHASE.of("@s")
    .transition()
    .from(BossPhase::Fighting)
    .when(HEALTH_PERCENT.of("@s").lte(50))
    .to(BossPhase::Enraged);
```

That emits the same `GameStateRef::set` representation, but it has no conflict
lock or hooks.

## Tick cadence and cost

`.on_tick(state, command)` explicitly means every server tick.
`.on_tick_every(state, Ticks::new(5), command)` uses a private scoreboard
counter that resets outside the matching state. Vanilla scheduling is
tick-granular; Sand does not offer sub-tick cadence.

One flow produces one selector scan per tick, one root helper, one helper per
transition, one helper per tick hook, one state equality check per transition
guard, and a private success-lock score. Interval hooks add one counter
objective and several score operations. Runtime cost therefore scales with
selected subjects, transitions, and tick hooks. For a very large selector or a
state machine already driven by a bespoke central tick function, the low-level
`GameStateRef`, `Condition`, and `TypedExecute` APIs may be cheaper and remain
fully supported.

StateFlow and tracked `SandEvent` transitions share the transition plan,
private-name collision checks, lifecycle merger, deterministic ordering, and
export path. StateFlow does not add an enum-value-as-type event API: Rust enum
values are not practical type parameters, and hooks already retain the
runtime subject as `@s`. Existing tracked events remain the event-composition
route when downstream event chaining is required.
