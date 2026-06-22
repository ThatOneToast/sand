# Sand Authoring Model

Sand's primary user-facing authoring model is ordinary Rust functions annotated
with Sand attributes.

## Primary Path

Use this path for normal datapack code:

- `#[function]` for named `.mcfunction` files.
- `#[component(Load)]` for commands that run from `minecraft:load`.
- `#[component(Tick)]` for commands that run from `minecraft:tick`.
- Typed command builders such as `cmd::tellraw`, `Actionbar::show`,
  `Title::of(...).build()`, `Sound::play(...)`, and `cmd::function(...)`.
- Typed state such as `ScoreVar<T>`, `Flag`, `Timer`, `Cooldown`, and
  `StorageVar<T>`.
- Typed conditions via `Condition`, `all!`, `any!`, and `TypedExecute`.
- Typed components for dialogs, recipes, advancements, loot tables, predicates,
  tags, and custom item data.

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));

#[component(Load)]
pub fn load() {
    MANA.define();
    DASH.define();
}

#[component(Tick)]
pub fn tick() {
    DASH.tick(Selector::all_players());
    TypedExecute::as_players()
        .when(all![MANA.of("@s").gte(25), DASH.ready("@s")])
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("Dash ready").aqua(),
        ));
}

#[function]
pub fn greet() {
    cmd::tellraw(
        Selector::all_players(),
        Text::new("Hello from Sand").gold(),
    );
}
```

This keeps rust-analyzer completion, Rust formatting, source-local compiler
errors, normal imports, and refactors working for datapack logic.

## Advanced Path

Use `mcfunction!` when you need command collection behavior that is intentionally
outside the beginner path:

- Grouping generated command fragments.
- Bridging APIs not modeled by Sand yet.
- Explicit raw interop with other datapacks, mods, or snapshot-only syntax.
- Macro-level control flow where typed builders are not enough.
- Compatibility with older Sand code while migrating toward attribute functions.

Raw commands should be explicit with `cmd::raw(...)`.

```rust
use sand_core::prelude::*;
use sand_macros::function;

#[function]
pub fn interop() {
    mcfunction! {
        cmd::raw("function other_pack:api/run");
        cmd::raw("execute as @a[tag=other_pack_ready] run function other_pack:tick");
    };
}
```

Raw command strings in render-output tests are fine. Raw command strings in
beginner docs or normal gameplay examples are not.
