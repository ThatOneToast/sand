# Unified State Components, Bundles, Queries, And Systems

`State` is Sand's canonical persistent gameplay-data declaration. Each schema
is an independently versioned component with one owner scope: `player`,
`entity`, `living`, or `global`. Its generated bound type has concrete named
fields, so normal Rust completion and navigation continue to work.

```rust
use sand::prelude::*;

#[derive(State)]
#[state(namespace = "trailforge", scope = living)]
pub struct Attack {
    #[state(default = 2, min = 0, max = 100)]
    pub damage: Score,
    #[state(auto_tick)]
    pub cooldown: Cooldown,
}

#[derive(State)]
#[state(namespace = "trailforge", scope = living)]
pub struct Defense {
    #[state(default = 0, min = 0, max = 100)]
    pub armor: Score,
}

#[derive(StateBundle)]
pub struct Combat {
    pub attack: Attack,
    pub defense: Defense,
}
```

Attach components through the generated lifecycle. Initialization fills only
missing owned values and publishes the component version last. Detachment runs
the cleanup hook first and removes only that component's fields and
bookkeeping. A bundle reuses its member components; it does not allocate a
second copy or tick a component twice.

```rust
#[function]
pub fn adopt_current_mob() {
    Combat::attach(EntityContext::<ZombieKind>::default());
}
```

Queries use ordinary named structs. Required and forbidden members become
selector presence filters. Optional members expose a generated callback whose
body is guarded by the actual runtime presence score; Sand does not fabricate
a compile-time `Option` from world state.

Required membership is selected when the Minecraft iteration begins. Optional
and forbidden guards are evaluated by the emitted `execute` commands, in body
order. Consequently, detaching a required component inside a callback does not
cancel commands already generated for that match, while an optional component
detached earlier in the same callback prevents its later guarded commands.

```rust
#[derive(StateQuery)]
#[query(scope = living)]
pub struct Fighters {
    #[require]
    pub combat: Combat,
    #[optional]
    pub status: Status,
    #[without]
    pub dead: Dead,
}

pub struct CombatSystems;

#[system]
impl CombatSystems {
    #[tick(every = 20)]
    fn regenerate(query: Fighters) {
        query.each(|fighter| fighter.combat.defense.armor.add(1));
    }
}
```

Player components automatically observe online players. Explicit player
detachment sets a suppression marker, so observation does not silently
reattach it; calling `attach` clears suppression. Entity and living schemas
never create an unconstrained adoption scan merely by being declared. Global
schemas use a deterministic singleton score holder and may additionally own
typed `Data<T>` paths in generated storage.

Version changes are explicit and contiguous. Declare each transition and use
an optional lifecycle implementation for transformation commands:

```rust
#[derive(State)]
#[state(
    namespace = "trailforge",
    scope = living,
    version = 2,
    migrate(from = 1, to = 2)
)]
pub struct Status {
    #[state(default = false)]
    pub poisoned: Flag,
}

#[state_lifecycle]
impl StateLifecycle for Status {
    fn migrate(ctx: StateMigrate) -> Vec<String> {
        vec![format!("say status {} -> {}", ctx.from(), ctx.to())]
    }
}
```

Changing a schema identity or generated objective naming is persisted-world
data migration, not a source-only rename. Preserve the old objective, copy or
transform its holder values in a declared migration, then remove it only after
the upgraded version marker is published. The complete compilable example is
in `examples/unified_state`.
