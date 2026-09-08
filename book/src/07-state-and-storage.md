# 7. Player State And Storage

Sand’s normal gameplay-data vocabulary is a scoped struct with
`#[derive(State)]`. The derive owns objective creation, default provisioning,
presence, lifecycle, and typed bound accessors.

```rust,ignore
use sand::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, StateEnum)]
enum Phase {
    Exploring,
    Fighting,
}

#[derive(State)]
#[state(namespace = "trail", scope = player)]
struct TrailState {
    #[state(default = 100, min = 0, max = 100)]
    stamina: Score,
    #[state(default = false)]
    has_striders: Flag,
    #[state(auto_tick)]
    grapple: Cooldown,
    #[state(default = "Phase::Exploring")]
    phase: EntityEnum<Phase>,
    #[state(default_snbt = "{}")]
    preferences: Data<serde_json::Value>,
}
```

Bind the schema to its scope and use the generated fields. There is no manual
objective registration step and no separate standalone variable to keep in
sync with the schema.

```rust,ignore
let state = TrailState::on(EntityContext::<PlayerKind>::default());
state.stamina.remove(10);
state.has_striders.enable();
state.grapple.start(Ticks::seconds(3));
state.phase.set(Phase::Fighting);
state.preferences.field("music").set(true);
```

Use `StateBundle` for reusable composition and `StateQuery` for presence-based
iteration. Systems and archetypes consume those same concrete State types.

## Command storage and world data

For data that is not owned by a State schema, use the canonical NBT model:
one `Nbt` root, an `NbtPath`, and an `NbtRef<T>`.

```rust,ignore
let range = Nbt::storage(ResourceLocation::new("trail", "data").unwrap())
    .typed_path::<i32>("config.grapple_range");
range.set(24);

let health = Nbt::entity(Target::self_()).typed_path::<f32>("Health");
health.get();
```

Minecraft command storage is global. Per-player structured data should normally
be a player-scoped State field; Sand does not pretend storage itself has native
per-player ownership.

Low-level scoreboard primitives remain under `sand::advanced::state` for
framework integrations. They are not a second application architecture.
