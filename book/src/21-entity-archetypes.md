# RPG Entities: State, Archetypes, And Derived Stats

This chapter builds the runnable `examples/rpg_entity` pack. It adopts loaded
Zombies, gives each one persistent typed state, derives combat stats, keeps
Minecraft properties synchronized, and reacts to state changes. The complete
source uses only the public facade:

```rust
use sand::prelude::*;
```

## Declare Entity State

The derive creates typed handles and stable per-entity scoreboard objectives.
The Rust struct is schema metadata; it is never constructed at runtime.

```rust
#[derive(EntityState)]
#[entity_state(namespace = "rpg", name = "zombie", version = 2)]
struct ZombieState {
    #[state(default = 1, min = 1, max = 100)]
    level: EntityScore<i32>,
    #[state(default = 20, min = 0, max = 2000)]
    health: EntityScore<i32>,
    #[state(default = 20, min = 1, max = 2000)]
    max_health: EntityScore<i32>,
    #[state(default = 3, min = 0, max = 1000)]
    attack_damage: EntityScore<i32>,
    #[state(default = false)]
    sick: EntityFlag,
    #[state(default = 0)]
    ability: EntityCooldown,
}
```

Typed operations emit commands against the entity currently bound to `@s`
and mark only their source field dirty:

```rust
ZombieState::level.bind().add(1);
ZombieState::sick.bind().enable();
ZombieState::ability.bind().start(Ticks::seconds(5));
```

These methods produce commands; they do not contact a server while Rust runs.

## Adopt Natural And External Zombies

`#[entity_archetype]` registers an immutable factory. Each export collects and
sorts fresh definitions, avoiding mutable process-global export state.

```rust
#[entity_archetype]
fn rpg_zombie() -> EntityArchetype<ZombieKind, ZombieState> {
    EntityArchetype::new(ResourceLocation::new("rpg", "plagued_zombie").unwrap())
        .adopt(
            Adoption::natural_and_external()
                .every(Ticks::new(5))
                .special_entities(SpecialEntityPolicy::Preserve),
        )
        .reconcile(ReconcilePolicy::WhenDirty)
}
```

The scan is constrained to `minecraft:zombie`. A Sand-owned content-hashed
tag makes initialization idempotent. Scans see loaded chunks only. Scoreboard
state stays attached across unloads, and reconciliation resumes after load.
Preserve mode changes only properties the archetype explicitly owns.

## Derive Health And Damage

Curves validate host constants, convert them to integers once, and use
fixed-point scoreboard arithmetic at runtime.

```rust
let fixed = FixedPoint::new(
    100,
    RoundingPolicy::TowardZero,
    OverflowPolicy::Error,
).unwrap();

let health = StatCurve::multiply([
    StatCurve::linear(
        StatCurve::state(ZombieState::level),
        2.0,
        18.0,
    ),
    StatCurve::enum_mapping(
        ZombieState::rarity,
        [(Rarity::Common, 1.0), (Rarity::Rare, 2.0), (Rarity::Legendary, 3.0)],
        1.0,
    ),
    StatCurve::flag_mapping(ZombieState::sick, 1.0, 0.75),
]);

let archetype = EntityArchetype::<ZombieKind, ZombieState>::new(
    ResourceLocation::new("rpg", "plagued_zombie").unwrap(),
)
.derive(
    EntityDerivation::new("max_health", ZombieState::max_health, health)
        .fixed_point(fixed),
);
```

A derivation stores a whole score by default; `store_fixed_point()` retains
scaled units. Curves support constants, affine and clamped-linear formulas,
addition, multiplication, ratios, steps, piecewise branches, lookups, enum
and flag maps, and canonical custom callbacks. Invalid ranges, non-finite
values, score overflow, duplicate targets, and dependency cycles stop export.

When `level`, `rarity`, or `sick` changes, only their transitive dependents are
recomputed.

## Synchronize Native Properties

Living-only methods are structurally available only for mutable living entity
kinds. Player entity-NBT writes are outside this capability surface.

```rust
archetype
    .health(
        HealthBinding::new(ZombieState::max_health)
            .current_health(ZombieState::health, CurrentHealthSync::Bidirectional)
            .resize(HealthResizePolicy::PreserveRatio)
            .observe_native_every(Ticks::new(20)),
    )
    .attribute(AttributeBinding::new(
        AttributeType::AttackDamage,
        NumericPropertySource::state(ZombieState::attack_damage),
    ));
```

The health bridge observes old current/max health, changes max health, and
preserves the ratio without healing on every reconciliation. Native health
observation pauses while unloaded. Function-macro arguments live under a
namespaced archetype path and are removed synchronously after every call, so
entities cannot retain one another's scratch values.

## Materialize A Colored Dynamic Name

```rust
let name = EntityText::new()
    .literal("Lv. ")
    .color_last(ChatColor::Gold)
    .score(ZombieState::level)
    .color_last(ChatColor::Yellow)
    .literal(" Plagued Zombie")
    .color_last(ChatColor::DarkGreen);

let archetype = archetype.name(NameBinding::new(name));
```

The generated function macro has a canonical resource ID and runs only after
one of its source fields becomes dirty.

## React To Sickness

```rust
#[function]
fn sickness_started() {
    ZombieState::stats_dirty.bind().set(1);
}

let archetype = archetype
    .effect_when(
        ZombieState::sick,
        EffectBinding::new(
            StatusEffectId::minecraft("weakness").unwrap(),
            Ticks::seconds(10),
        ),
    )
    .on(
        EntityTransition::flag_enabled(ZombieState::sick),
        EntityAction::Run(FunctionRef::new("rpg:sickness_started").unwrap()),
    );
```

Transition history is per entity. The action function runs with that entity
bound to `@s`, composing with Sand's existing functions, events, state, VFX,
summons, tags, equipment, and typed property APIs.

## Reconcile And Migrate

```rust
#[function]
fn migrate_v1_v2() {
    ZombieState::schema_version.bind().set(2);
}

let archetype = archetype
    .version(2)
    .migration(Migration::new(
        1,
        2,
        FunctionRef::new("rpg:migrate_v1_v2").unwrap(),
    ));
```

Migrations must form a contiguous path to the current version. Reconciliation
provisions new state first, runs migrations in order, refreshes only declared
owned properties, and records completion last. Missing paths and conflicting
property owners are structured export errors.

Vanilla has no callback for every unload or external removal, so cleanup is
best effort. Explicitly call the generated cleanup function while an entity is
loaded when teardown is required.

## Run It

```text
cargo test --manifest-path examples/rpg_entity/Cargo.toml
cd examples/rpg_entity
SAND_EXPORT_MC_VERSION=26.2 cargo run --bin sand_export
```

The test requires two exports to be byte-identical. The Minecraft 26.2
validation harness installs the pack, reloads it, summons an unmarked Zombie
through the same adoption path used by natural Zombies, changes level, checks
derived attributes, ratio-preserved health, the dynamic name, migration,
scratch cleanup, and removal, and saves the server log as runtime evidence.
