---
id: basic-pack
capabilities:
  - functions
  - load-tick-components
  - score-state
  - text-and-ui
minecraft:
  minimum: "1.18.0"
  maximum_verified: "26.2.0"
cargo_features: []
verification:
  compiles: true
  golden_output: false
  vanilla_reload: false
---

# Basic pack

## Intent

A minimal datapack: load hook, a persistent counter, and a callable function
that greets the player.

## Required crates and features

`sand-core` (no optional `systems-*` features needed), `sand-macros`. Default
import: `sand_core::prelude::*`.

## Code

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

static VISITS: ScoreVar<i32> = ScoreVar::new("visits");

#[component(Load)]
pub fn load() {
    VISITS.define();
    cmd::tellraw(Selector::all_players(), Text::new("Pack loaded").green());
}

#[function]
pub fn greet() {
    VISITS.add(Selector::self_(), 1);
    cmd::tellraw(Selector::self_(), Text::new("Hello from Sand").gold().bold(true));
}
```

## Expected generated resources

- `data/<namespace>/function/load.mcfunction` — registered into
  `data/minecraft/tags/function/load.json` by `#[component(Load)]`.
- `data/<namespace>/function/greet.mcfunction` — plain callable function
  (`/function <namespace>:greet`).
- `scoreboard objectives add visits dummy` emitted by `VISITS.define()`
  inside `load.mcfunction`.

## Sand limitations

None for this scope — functions, load components, `ScoreVar`, and `Text`
are all `implemented`/`stable` per `ai/capability-manifest.yaml`.

## Vanilla limitations

None — this is core datapack functionality with no vanilla constraint.

## Validation steps

1. `cargo build` in the scaffolded project.
2. `cargo run -p sand -- build`, then read `dist/<pack>/data/<namespace>/function/load.mcfunction` and confirm it contains the `scoreboard objectives add` and `tellraw` lines.
3. Not vanilla-reload-verified in this review — see `ai/known-limitations.md` (`LIM-VAL-001`).

## Common incorrect approaches

- Writing `/scoreboard objectives add visits dummy` as a raw string via
  `cmd::raw(...)` — `ScoreVar::define()` already generates this; using raw
  commands here bypasses the typed API for no reason (see
  `ai/authoring-guide.md` anti-hallucination rules).
- Calling `VISITS.add(...)` before `VISITS.define()` has run at least once —
  the objective must exist before any command mutates it; keep `define()` in
  a `#[component(Load)]` function so it runs on every `/reload`.
