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

`#[entity_archetype]` registers an immutable factory. The only type parameter
is the Minecraft entity kind:

```rust
#[entity_archetype]
fn seeker() -> EntityArchetype<ZombieKind> {
    EntityArchetype::new("rpg:seeker".parse().unwrap())
        .components::<Progression>()
        .components::<Combat>()
        .components::<Conditions>()
        .adopt(Adoption::natural_and_external().every(Ticks::new(5)))
        .reconcile(ReconcilePolicy::WhenDirty)
}
```

`.components::<B>()` also accepts a nested `StateBundle`. Sand flattens bundles
in declaration order and deduplicates repeated component identities. It does
not merge schemas: every component continues to own its storage and lifecycle.

The adoption scan remains constrained to `minecraft:zombie`. A Sand-owned
marker makes initialization idempotent. Scans see loaded chunks only, while
scoreboard state survives unloading and reconciliation resumes after load.

## Derive Across Components

The normal derivation API takes a typed target and a curve. Its stable identity
and target encoding come from the State field metadata:

```rust
let archetype = seeker().derive(
    Combat::max_health,
    StatCurve::linear(StatCurve::state(Progression::level), 2.0, 18.0),
);
```

Inputs and targets may belong to different composed components. Chained
cross-component derivations are sorted by dependency, dirty changes propagate
to later targets, and a real cycle across components stops export. A target or
input from an unattached component also stops export with the archetype,
property, component, and field in the diagnostic.

For deliberate fixed-point or output-encoding overrides, construct the
advanced value explicitly:

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
introduce another scaling convention.

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
