---
id: event-driven-feature
capabilities:
  - advancement-triggers
  - events-typed
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

React to a gameplay event using an advancement trigger, with a typed
function reward, and re-arm the detector by revoking the advancement. This
is the standard pattern for "detect X and run a function" when no dedicated
Sand systems helper exists for X (e.g. join detection —
`systems-lifecycle`'s `lifecycle-events` capability wraps this exact pattern
for join/death/respawn; reach for it first if it fits, and use this raw
pattern for other trigger-shaped events it doesn't cover).

## Required crates and features

`sand-core`, `sand-macros`. No optional Cargo features required for the
`minecraft:tick`-trigger pattern shown here.

## Code

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

static VISITS: ScoreVar<i32> = ScoreVar::new("visits");

#[component(Load)]
pub fn on_load() {
    VISITS.define();
}

#[component]
pub fn detect_join() -> Advancement {
    Advancement::new(ResourceLocation::new("my_pack", "detect_join").unwrap())
        .criterion("joined", Criterion::new(AdvancementTrigger::Tick))
        .rewards(AdvancementRewards::new().function("my_pack:on_player_join"))
}

#[function]
pub fn on_player_join() {
    VISITS.add(Selector::self_(), 1);
    cmd::tellraw(Selector::self_(), Text::new("Welcome back").gold());
    cmd::advancement_revoke_only(Selector::self_(), "my_pack:detect_join");
}
```

## Expected generated resources

- `data/my_pack/advancement/detect_join.json` — a `minecraft:tick`-triggered
  advancement whose reward function is `on_player_join`.
- `data/my_pack/function/on_player_join.mcfunction` — increments `visits`,
  sends a message, then runs `advancement revoke @s only my_pack:detect_join`
  to re-arm detection for the next login.
- `data/my_pack/function/load.mcfunction` — defines the `visits` objective.

## Sand limitations

None — advancement triggers and typed function refs are `implemented`.

## Vanilla limitations

Advancement triggers fire on a condition check, not a rich event with a
payload (`LIM-VAN-002` in `ai/known-limitations.md`). The
`minecraft:tick`-trigger-plus-revoke pattern above is the standard vanilla
idiom for detecting joins because there is no dedicated "on join" trigger;
it works by re-arming every tick per player and firing exactly once per
login, but it is a polling approximation, not a native event.

## Validation steps

1. `cargo build`.
2. `cargo run -p sand -- build`; read `dist/.../advancement/detect_join.json` and confirm the trigger/reward shape, and `on_player_join.mcfunction` for the revoke command.
3. Not vanilla-reload-verified in this review — confirming the re-arm timing (exactly once per login, not once per tick) requires an actual server test.

## Common incorrect approaches

- Omitting the `advancement_revoke_only` call — without it, the advancement
  stays granted and `on_player_join` never fires again for that player.
- Assuming `AdvancementTrigger::Tick` exposes any event payload — it's a
  pure condition check; all context (who joined, what to do) has to come
  from `Selector::self_()` and typed state you read/write yourself.
- Reaching for a `systems-lifecycle` type without enabling that Cargo
  feature — if the project doesn't need the fuller lifecycle helpers, this
  raw advancement pattern needs no extra feature flags at all.
