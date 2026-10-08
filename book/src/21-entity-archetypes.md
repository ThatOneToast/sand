# RPG Entities: Components, Archetypes, And Derived Stats

An entity archetype is a named composition of reusable State components plus
native Minecraft behavior. State owns data and lifecycle, the archetype owns
composition and native bindings, and systems provide behavior over matching
components. There is no primary or root State.

This chapter follows the runnable `examples/rpg_entity` pack using the public
facade:

```rust
use sand::prelude::*;
```

## Declare Reusable Components

Each `State` has its own stable identity, objectives, presence, version,
migrations, and attach/detach lifecycle. The structs are schema metadata; they
are not runtime entity objects.

```rust
#[derive(State)]
#[state(namespace = "rpg", scope = living)]
struct Progression {
    #[state(default = 1, min = 1, max = 100)]
    level: Score,
}

#[derive(State)]
#[state(namespace = "rpg", scope = living)]
struct Combat {
    #[state(default = 20, min = 0, max = 2000)]
    health: Score,
    #[state(default = 20, min = 1, max = 2000)]
    max_health: Score,
    #[state(default = 3, min = 0, max = 1000)]
    attack_damage: Score,
}

#[derive(State)]
#[state(namespace = "rpg", scope = living)]
struct Conditions {
    #[state(default = false)]
    sick: Flag,
}
```

Typed writes target the entity bound to `@s` and mark that component field
dirty:

```rust
Progression::level.bind().add(1);
Conditions::sick.bind().enable();
```

Components can be attached directly before adoption. Archetype initialization
uses the same idempotent component lifecycle, so it fills only missing values
and never resets a valid value or creates a second copy.

## Compose The Archetype

Declare the gameplay object as a Rust type. Its fields name the existing
components, and its attribute declares the identity and entity kind once:

```rust
#[derive(Archetype)]
#[archetype(id = "rpg:seeker", entity = Zombie, configure = Self::configure)]
pub struct Seeker {
    pub progression: Progression,
    pub combat: Combat,
    pub conditions: Conditions,
}

impl Seeker {
    fn configure(archetype: EntityArchetype<ZombieKind>) -> EntityArchetype<ZombieKind> {
        archetype
            .adopt(Adoption::natural_and_external().every(Ticks::new(5)))
            .reconcile(ReconcilePolicy::WhenDirty)
    }
}
```

Fields also accept nested `StateBundle` types and marker State. Each flattened
component retains its own storage and lifecycle. Repeating a component across
fields is rejected with the declaration and conflicting field names.

The optional `configure` callback receives the already composed definition.
It uses ordinary Rust to attach native bindings, curves, migrations, and policy.
It must return that declaration with the same identity and component list;
export rejects replacement identities or extra components. Omit `configure`
when no native behavior is needed. This callback keeps the existing typed
builder available without requiring a second configuration trait or repeating
identity and composition in every behavior implementation.

`#[entity_archetype]` and `EntityArchetype<K>` remain available for advanced
programmatic definitions. The concrete declaration is the normal authoring
entry point.

The adoption scan remains constrained to `minecraft:zombie`. A Sand-owned
marker makes initialization idempotent. Scans see loaded chunks only, while
scoreboard state survives unloading and reconciliation resumes after load.

## Create, Adopt, And Access A Concrete Object

```rust
#[function]
fn spawn_seeker() {
    Seeker::summon(Vec3::here());
}

#[function]
fn adopt_nearby() {
    Seeker::adopt(Target::entities().within_blocks(32.0));
}

#[system(tick, every = 20)]
fn level_seekers(query: Seeker) {
    query.each(|seeker| seeker.progression.level.add(1));
}
```

The prelude imports `ArchetypeOperations`, which supplies these concrete-type
operations. Summoning requires `SummonableEntityKind` and initializes only the
newly created entity. Player archetypes can adopt existing players but cannot
summon them. Explicit adoption keeps
the caller's selection and initializes matching Zombies at their own positions;
a conflicting entity filter selects nothing rather than being overwritten.
Both paths use canonical initialization, migrations, and native bindings.

An archetype system query selects its entity kind, membership marker, and
required component presence. It uses the same query lowering as `StateQuery`.
`query.current(...)` checks the current executor without changing it.

`Seeker::on(entity)` returns a named `SeekerBound` view with `progression`,
`combat`, and `conditions` fields. `Seeker::attach(entity)`, `detach(entity)`,
and `is_attached(entity)` accept the declared kind's `EntityContext`. Detaching
preserves a component retained by another archetype. `on` does not attach
missing State, and `is_attached` returns a runtime `Condition`.

These handles address the current Minecraft executor. They are not persistent
Rust references and must not be retained across an executor change. Use
`seeker.entity()` to access that executor's typed capabilities.

## Derive Across Components

The normal derivation API takes a typed target and a curve. Its stable identity
and stored representation come from the State field metadata:

```rust
let archetype = archetype.derive(
    Combat::max_health,
    StatCurve::linear(StatCurve::state(Progression::level), 2.0, 18.0),
);
```

Inputs and targets may belong to different composed components. Chained
cross-component derivations are sorted by dependency, dirty changes propagate
to later targets, and a real cycle across components stops export. A target or
input from an unattached component also stops export with the archetype,
property, component, and field in the diagnostic.

Curves are written in logical gameplay units. A `Score` target receives a whole
number, while a `FixedScore` target receives its declared scale. Inputs may use
different scales; `StatCurve::state` carries that metadata into the same
expression and Sand resizes it automatically. For deliberate working-precision
or rounding-policy overrides, construct the advanced value explicitly:

```rust
let fixed = FixedPoint::new(
    100,
    RoundingPolicy::TowardZero,
    OverflowPolicy::Error,
).unwrap();
let health_curve = StatCurve::state(Progression::level);

let archetype = archetype.derive_with(
    EntityDerivation::for_target(Combat::max_health, health_curve)
        .fixed_point(fixed),
);
```

This uses the same numeric model as ordinary `StatCurve` lowering; it does not
introduce another scaling convention. The complete scale and rounding model is
covered in [Numeric State: scales, rounding, and arithmetic](23a-numeric-state.md).

## Bind Native Minecraft Behavior

Every State-backed archetype property resolves through the flattened
composition. Health can use Combat while a conditional effect uses Conditions:

```rust
let archetype = archetype
    .health(
        HealthBinding::new(Combat::max_health)
            .current_health(Combat::health, CurrentHealthSync::Bidirectional)
            .resize(HealthResizePolicy::PreserveRatio)
            .observe_native_every(Ticks::new(20)),
    )
    .attribute(AttributeBinding::new(
        AttributeType::AttackDamage,
        NumericPropertySource::state(Combat::attack_damage),
    ))
    .effect_when(
        Conditions::sick,
        EffectBinding::new(
            StatusEffectId::minecraft("weakness").unwrap(),
            Ticks::seconds(10),
        ),
    );
```

The same membership rule covers attributes, modifiers, equipment, tags,
teams, transitions, adoption predicates, and names wherever those existing
properties accept typed State fields.

## Build A Dynamic Name With Canonical Text

Static segments use Sand's normal `Text`/`TextComponent` styling. Dynamic
segments take a typed State field and their own color, so styling is applied as
the segment is authored:

```rust
let name = EntityName::new()
    .text(Text::new("Seeker Lv. ").gold())
    .state(Progression::level, ChatColor::Yellow)
    .text(Text::new(" [").gray())
    .state(Combat::health, ChatColor::Red)
    .text(Text::new("/").gray())
    .state(Combat::max_health, ChatColor::Red)
    .text(Text::new("]").gray())
    .refresh_every(Ticks::new(5));

let archetype = archetype.name(name);
```

Enum and flag segments use `enum_state` and `flag_state`. Sand keeps the
archetype-specific score materialization behind this shared authoring model.
A State field from an unattached component receives the same membership
diagnostic as any other archetype property.

## Lifecycle And Migration

Component migrations remain declared on each `State`. Archetype migrations
version changes to the composition or its native behavior:

```rust
let archetype = archetype
    .version(2)
    .migration(Migration::new(
        1,
        2,
        "rpg:migrate_v1_v2".parse::<FunctionId>().unwrap(),
    ));
```

Initialization orchestrates canonical component attachment before native
setup and publishes the archetype marker last. Cleanup runs the optional
archetype callback, detaches its flattened components through their canonical
lifecycle, and leaves unrelated components alone. Two archetypes can reuse the
same State type without changing that component's identity or storage model.

Vanilla has no callback for every unload or external removal, so cleanup is
best effort. Explicitly call the generated cleanup function while an entity is
loaded when teardown is required.

## Run It

```text
cargo test --manifest-path examples/rpg_entity/Cargo.toml
cd examples/rpg_entity
SAND_EXPORT_MC_VERSION=26.2 cargo run --bin sand_export
```

The example test requires two exports to be byte-identical. The Minecraft
validation harness installs the pack, reloads it, adopts an unmarked Zombie,
changes level, checks derived attributes, ratio-preserved health, the dynamic
name, migration, scratch cleanup, and removal.
