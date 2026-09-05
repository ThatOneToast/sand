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

Attach or adopt the component, then query it directly. A scoped `State` is the
one-component query for owners that have its real presence/version marker; the
callback receives the normal bound view, so its fields are immediately
available:

```rust
#[derive(State)]
#[state(namespace = "trailforge", scope = living)]
pub struct Health {
    #[state(default = 20, min = 0, max = 100)]
    pub current: Score,
}

#[system(tick, every = 20)]
fn regenerate(query: Health) {
    query.each(|health| health.current.add(1));
}

#[system(tick, every = 20)]
fn train_combat(query: Combat) {
    query.each(|combat| combat.attack.damage.add(1));
}
```

`query.current(...)` performs the same presence check against an executor that
an event dispatcher has already selected; it never starts another scan. A
`StateBundle` can likewise be a direct query, requiring every flattened member
and yielding its normal nested bound view.

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
anything twice. Bundle operations keep the same owner rules as their members:
a player bundle only accepts a player context, a living bundle only accepts a
living context, and detaching a player bundle opts every member out of automatic
re-observation until it is attached again.

```rust
#[function]
pub fn adopt_current_mob() {
    Combat::attach(EntityContext::<ZombieKind>::default());
}
```

Global bundles do not borrow the current executor at all. Each member keeps
its own deterministic singleton holder, even when several resources are
grouped behind one view:

```rust,ignore
let world = WorldResources::global();
world.progression.wave.add(1);
WorldResources::attach_global();
```

Use `StateQuery` once a system needs composition. Required and forbidden
members become selector filters. Optional members use a callback guarded by the
real runtime presence score—there is no pretend Rust `Option` decided while the
pack is being built. Sand flattens nested bundles while checking the
declaration, so a query cannot require a bundle and forbid one of that bundle's
components. A one-field required `StateQuery` is unnecessary; query that State
directly instead.

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

## Typed system authoring

System parameters are the concrete types written in source. With
`sand::prelude::*` imported, `query.` completes to `each` and `current` on a
direct scoped `State`, a `StateBundle`, or a composed `StateQuery`. Completion
inside the callback sees the generated bound State fields or the named query
item fields. There is no generic tuple or hidden handle to write in an author
signature.

A free system is a tick system and takes exactly one simply named query
parameter. Empty `#[system]` means every tick; spelling `tick` is optional but
can make intent clearer. `every` is a positive tick count. Keep the query
parameter name unshadowed within the body—including by local value items or
imports; Sand uses that binding to identify the typed query operations lowered
into the export adapter:

```rust
#[system]
fn every_tick(query: Health) {
    query.each(|health| health.current.add(1));
}

#[system(tick, every = 20)]
fn every_second(query: Health) {
    query.each(|health| health.current.add(1));
}
```

`cfg` and `cfg_attr` gates on a free system, grouped impl, or grouped method
also gate its generated registration. Statement attributes on `query.each(...)`
and `query.current(...)` are preserved in the export adapter.

An inherent impl groups related endpoints without constructing a Rust system
object. Grouped methods do not take `self`. Tick methods use `#[tick]` or
`#[tick(every = N)]`:

```rust
pub struct HealthSystems;

#[system]
impl HealthSystems {
    #[tick]
    fn update(query: Fighters) {
        query.each(|fighter| fighter.combat.defense.armor.add(1));
    }

    #[tick(every = 20)]
    fn regenerate(query: Health) {
        query.each(|health| health.current.add(1));
    }
}
```

Event endpoints exist in the grouped form. Their first parameter must have the
same type named by `#[event(...)]`; an optional query is second. Dispatch has
already bound the event owner as `@s`, so `current` guards commands with the
query's required and forbidden presence checks without selecting entities a
second time. Calling `each` is valid when the event really needs a new
scope-wide search, but it creates that second scan.

```rust,ignore
#[system]
impl CombatSystems {
    #[event(PlayerAttack)]
    fn attack(_event: PlayerAttack, query: Fighters) {
        query.current(|owner| owner.combat.attack.damage.add(1));
    }
}
```

System and query closures run in Rust while Sand builds the datapack. Their
result is an ordered `Vec<String>` of Minecraft commands, not an in-memory
mutation of an entity. A single typed State operation already returns that
vector. To combine operations, collect or extend their vectors and return the
combined value from the closure:

```rust
query.each(|fighter| {
    let mut commands = fighter.combat.attack.damage.add(1);
    commands.extend(fighter.status(|status| status.poison_time.tick()));
    commands
});
```

At export, Sand sorts systems by their stable Rust module/type/function
identity. That order determines command order. Consecutive systems in that
order share a scan only when they have the same cadence and the complete body
of each is the exact typed query scan with the same selector. A different
scope, presence filter, cadence, mixed selector, or opaque command keeps a
separate system body. Generated objective and function names derive from the
stable identities, collision-check deterministically, and identical dynamic
callback bodies are deduplicated.

Minecraft selectors visit loaded entities only. Sand does not load chunks and
does not maintain an in-memory Rust ECS between ticks. Required component
membership is captured by the outer selector when an `each` scan begins.
Optional and forbidden membership is checked by generated commands in their
emitted order. Therefore detaching a required component during a callback does
not remove the owner from the scan already in progress, while attaching or
detaching a component can change a later optional/forbidden guard in that same
ordered command sequence.

Archetypes compose independent components and nested bundles with
`.components::<Combat>()`. Composition uses the same idempotent attachment,
component migration, and ownership-safe detachment paths as direct author
calls. An archetype has no primary State: it owns the flattened composition and
native Minecraft bindings while every component retains its own data and
lifecycle.

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
