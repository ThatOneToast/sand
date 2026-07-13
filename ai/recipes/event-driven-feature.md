---
id: event-driven-feature
capabilities:
  - events-typed
  - advancement-triggers
  - score-state
minecraft:
  minimum: "1.18.0"
  maximum_verified: "26.2.0"
cargo_features: []
verification:
  compiles: true
  golden_output: false
  vanilla_reload: false
---

# Event-driven feature

## Intent

React to a gameplay event with a typed handler. Sand ships a built-in event
library (`sand_core::events`) covering joins, deaths, respawns, combat,
item use, block placement, equipment changes, and more — reach for one of
those with `#[event]` before hand-rolling an advancement. Only build a raw
advancement-trigger-plus-revoke component when the event isn't in that
built-in list and isn't expressible as a custom `AdvancementEvent`/`SandEvent`
either.

## Required crates and features

`sand-core`, `sand-macros`. No optional Cargo features required for either
pattern shown here.

## Code — preferred: built-in typed event

```rust
use sand_core::events::OnJoinEvent;
use sand_core::prelude::*;
use sand_macros::{event, function};

static VISITS: ScoreVar<i32> = ScoreVar::new("visits");

#[function]
pub fn welcome_back() {
    VISITS.add(Selector::self_(), 1);
    cmd::tellraw(Selector::self_(), Text::new("Welcome back").gold());
}

#[event]
pub fn on_join(event: Event<OnJoinEvent>) {
    let _ = event;
    cmd::call(welcome_back);
}
```

`#[event]` on a handler taking `Event<OnJoinEvent>` generates all detection
and dispatch — no `#[component(Load)]`, advancement JSON, or manual
`advancement revoke` call needed. See `sand_core::events`'s module docs for
the full built-in list (`OnDeathEvent`, `OnRespawnEvent`, `ItemConsumeEvent`,
`BlockPlaceEvent`, `ArmorEquipEvent`, `EntityKillEvent`, and ~40 more), and
`sand_core::event::AdvancementEvent`/`sand_core::events::SandEvent` for
defining your own typed event backed by a specific advancement trigger or
tick condition.

## Code — fallback: raw advancement trigger (only when no typed event fits)

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

static VISITS: ScoreVar<i32> = ScoreVar::new("visits");

#[component(Load)]
pub fn on_load() {
    VISITS.define();
}

#[component]
pub fn detect_custom_condition() -> Advancement {
    Advancement::new(ResourceLocation::new("my_pack", "detect_custom_condition").unwrap())
        .criterion("met", Criterion::new(AdvancementTrigger::Tick))
        .rewards(AdvancementRewards::new().function("my_pack:on_custom_condition"))
}

#[function]
pub fn on_custom_condition() {
    VISITS.add(Selector::self_(), 1);
    cmd::tellraw(Selector::self_(), Text::new("Condition met").gold());
    cmd::advancement_revoke_only(Selector::self_(), "my_pack:detect_custom_condition");
}
```

This is the same mechanism the built-in events are generated from — use it
directly only for a condition none of them cover (or implement
`SandEvent`/`AdvancementEvent` on your own marker type and keep using
`#[event]`, which gets you the same self-registration ergonomics).

## Expected generated resources

**Built-in event (`OnJoinEvent`) version:**
- `data/my_pack/function/on_join.mcfunction` — the handler body (`function
  my_pack:welcome_back`).
- `data/my_pack/function/__sand_join_check.mcfunction` — Sand-generated,
  registered in `minecraft:tick`, dispatches to `on_join` for any online
  player whose `__sand_join` score isn't `1`, then sets it.
- `data/my_pack/function/__sand_join_init.mcfunction` — Sand-generated,
  registered in `minecraft:load`, defines and resets the `__sand_join`
  objective.

**Raw advancement fallback version:**
- `data/my_pack/advancement/detect_custom_condition.json` — a
  `minecraft:tick`-triggered advancement whose reward function is
  `on_custom_condition`.
- `data/my_pack/function/on_custom_condition.mcfunction` — increments
  `visits`, sends a message, then runs `advancement revoke @s only
  my_pack:detect_custom_condition` to re-arm detection.
- `data/my_pack/function/load.mcfunction` — defines the `visits` objective.

## Sand limitations

None — the built-in event library, advancement triggers, and typed function
refs are all `implemented`.

## Vanilla limitations

`OnJoinEvent` fires on the first tick after each server start/reload, or for
a new player mid-session — but a mid-session **disconnect → reconnect**
without a `/reload` does **not** re-fire it, because the join flag persists
in `scoreboard.dat` (see `sand_core::events::OnJoinEvent`'s docs). True
per-login detection for reconnects requires a mod or plugin; it is not
achievable in vanilla datapacks. For a "very first join, ever" welcome
instead of "every session," use `Event<FirstJoinEvent>`.

More generally, advancement triggers fire on a condition check, not a rich
event with a payload (`LIM-VAN-002` in `ai/known-limitations.md`) — this
applies to both the built-in events and the raw fallback pattern.

## Validation steps

1. `cargo build`.
2. `cargo run -p sand -- build`; for the `OnJoinEvent` version, read
   `dist/.../function/on_join.mcfunction`, `__sand_join_check.mcfunction`,
   and `__sand_join_init.mcfunction`. For the raw fallback, read
   `dist/.../advancement/detect_custom_condition.json` and
   `on_custom_condition.mcfunction`.
3. Verified against a real `sand new --path-deps && sand build` run with
   network access (2026-07-12) for the `OnJoinEvent` pattern — confirmed
   the three generated files above and their contents. The raw-advancement
   fallback was compile-checked only, not re-verified against a live build
   in this pass. Neither was vanilla-reload-verified (`LIM-VAL-001`).

## Common incorrect approaches

- Hand-writing an advancement + `#[component(Load)]` + revoke call for join
  detection — `OnJoinEvent` already generates this; duplicating it is dead
  weight and easy to get subtly wrong (e.g. forgetting the revoke, or using
  a tag instead of a scoreboard and getting the wrong persistence
  semantics — see the vanilla-limitations note above).
- Omitting `advancement_revoke_only` in the raw fallback pattern — without
  it, the advancement stays granted and the reward function never fires
  again for that player.
- Assuming `AdvancementTrigger::Tick` or a built-in event exposes rich event
  payload data — both are pure condition checks; all context has to come
  from `Selector::self_()`/`event.player()` and typed state you read/write
  yourself.
