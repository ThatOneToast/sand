# State without the bookkeeping

`State` is where long-lived gameplay data goes. You write a normal Rust struct,
pick who owns it (`player`, `entity`, `living`, or `global`), and Sand handles
the scoreboard names, initialization, presence marker, and version. The view
you use in gameplay code still has real named fields, so completion and “go to
definition” work as you would expect.

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

Use `FixedScore` when whole numbers are too chunky. Its default scale is 1,000,
or you can choose one explicitly. This stores `1.25` as `125`, rounds exact
halves away from zero, and clamps the stored result to the declared bounds.
An `add` or `subtract` argument is a delta: Sand scales it directly, then
clamps the value after applying it. In other words, a minimum of `1.0` does
not turn `add(0.10)` into `add(1.0)`:

```rust
#[derive(State)]
#[state(namespace = "trailforge", scope = player)]
pub struct Movement {
    #[state(default = 1.25, min = 0, max = 8, scale = 100)]
    pub speed: FixedScore,
}

let movement = Movement::on(EntityContext::<PlayerKind>::default());
movement.speed.add(0.10);
```

Attaching is safe to repeat. Sand only fills in missing values and publishes
the version marker after everything else succeeds. Detaching runs cleanup and
removes that component's own values, leaving other components alone. Bundles
are just named views over their members; they do not make a second copy or tick
anything twice.

```rust
#[function]
pub fn adopt_current_mob() {
    Combat::attach(EntityContext::<ZombieKind>::default());
}
```

Queries are ordinary named structs too. Required and forbidden members become
selector filters. Optional members use a callback guarded by the real runtime
presence score—there is no pretend Rust `Option` decided while the pack is
being built.

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

Event systems can take the query after the event and call `current`. The event
dispatcher has already made the owner `@s`, so Sand checks the component
filters without scanning the world again. Tick systems with the same cadence
and selector share a scan when they are next to each other in deterministic
system order; opaque command bodies stay separate.

```rust,ignore
#[event(PlayerAttack)]
fn attack(event: PlayerAttack, query: Fighters) {
    query.current(|owner| owner.combat.attack.damage.add(1));
}
```

Archetypes compose independent components and nested bundles with
`.components::<Combat>()`. Composition uses the same idempotent attachment,
component migration, and ownership-safe detachment paths as direct author
calls. Use a distinct marker as the archetype's primary schema; repeating that
schema inside a composed bundle is rejected as a conflicting policy.

Player components watch online players automatically. If you explicitly detach
one, Sand remembers that choice instead of quietly putting it back; `attach`
opts the player in again. This also applies to typed data: detaching removes
the component's keyed storage, and the automatic player observer will leave it
gone until you attach the component again. Entity and living components never
start a world-wide scan just because their type exists. Global state uses one
deterministic holder and can also own typed `Data<T>` paths in command storage.

When a typed field changes, Sand marks both that field's archetype work and the
component's lifecycle reconciliation as dirty. Those jobs are independent, so
one cannot accidentally consume the other's update. A registered `reconcile`
hook runs once for the dirty component on an eligible loaded owner, after which
Sand clears the component marker. This keeps native properties in step with
state without ticking unrelated components or starting an unbounded scan.

`Data<T>` works for scoped components too. Sand keys those values by the
current player's or entity's UUID in command storage. That means the data
survives unloads without being stuffed into unreliable custom entity NBT. Use
the generated `if_present` callback when existence matters at runtime:

```rust
let progression = Progression::on(EntityContext::<PlayerKind>::default());
progression.preferences.if_present(|| vec!["say welcome back".into()]);
```

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

One last gotcha: changing a schema name or an objective identity changes saved
world data. Treat that like a database rename. Keep the old value around, copy
or transform it in a migration, and remove it only after the new version marker
has been published. A complete project lives in `examples/unified_state`.
