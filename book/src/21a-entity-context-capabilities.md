# Entity Context Capabilities

`EntityContext<K>` represents the entity currently executing as `@s` in one
generated command chain. It is not a UUID, selector, or reference that can be
stored across ticks. When a relationship traversal temporarily changes `@s`,
use `EntityScope::bind` to keep addressing the original entity for the duration
of that generated chain.

Focused handles keep related operations easy to find:

- `identity()` provides validated tags, teams, and kill semantics.
- `transform()` provides position and rotation reads plus teleport and facing.
- `living()` provides health, damage, effects, and attributes.
- `equipment()` returns Sand's existing typed `ItemLocation` values.
- `inventory()` is available for known players and returns Sand's existing
  selector-preserving `EntityInventory` factory.
- `mounts()` provides ride and dismount mutations; relationship methods remain
  the canonical way to observe vehicles and passengers.
- `data()` reads typed entity NBT and permits writes only for entity kinds where
  vanilla supports them safely.

The kind parameter is a compile-time capability boundary. For example,
`EntityContext<MarkerKind>` has no `living()` or `equipment()` method, while
`EntityContext<PlayerKind>` has both but cannot call the mutable methods on an
`EntityData` path. Live item locations expose a `ReadOnlyNbtRef`, so player
inventory changes must use typed `/item` operations rather than entity-NBT
mutation. An `AnyEntity` context from `Target::entities()` exposes
only operations valid without knowing the selected entity's concrete type.
Event `PlayerParticipant` and `EntityParticipant` references expose the same
capabilities that their known kind permits while retaining the participant's
selector, reliability, and lifetime contract.

This example imports only `sand::prelude::*` and combines query iteration,
identity, transform, living, and equipment capabilities:

```rust
{{#include ../../examples/book_project/src/lib.rs:entity_capabilities}}
```

`data().field::<T>(...)` is the preferred path when the vanilla field is known.
`raw_path(...)` is deliberately conspicuous: use it only for unsupported or
third-party NBT layouts, and never to bypass a State/native-property binding.
Archetype health, attribute, equipment, and other property bindings remain the
owners of synchronization and lifecycle behavior. Obtaining a handle creates
no gameplay State or background scans. Compiled queries can own private helper
functions, predicates, and temporary numeric scores; they do not install
watchers or take over archetype property ownership.

All of these APIs target Sand's supported Minecraft 26.x+ command model.

## Effect presence

Use `living().has_effect` to compose an immediate native effect query with
inventory, numeric, or State Conditions:

```rust,ignore
Target::players().each(|player| {
    let speedy = player.living().has_effect(EffectId::Speed);
    when(speedy.and(player.inventory().contains(vanilla::Item::Diamond)))
        .then_all([player.living().clear_effect(EffectId::Slowness)])
});
```

This reuses Sand's typed effect identifier and entity predicate model. Equivalent
queries share a predicate resource, and cached actions retain it on every export.
The owner is selected once. A missing entity or absent effect makes the query
false; negation also matches missing entities. This creates no polling loop or
persistent observation State. Use `give_effect`, `clear_effect`, and
`clear_effects` for native effect mutations.

## Native damage and effect actions

Living capabilities produce authored operations: `damage`, `clear_effect`, and
`clear_effects` return `Actions`. `give_effect` returns the canonical `EffectGive`
builder, which function macros, `when(...).then(...)`, and
`Execute::run_actions` collect without converting it to text. Player handles
injected into functions or obtained from events use the same living capability:

```rust,ignore
#[function("combat:dispel", context = player)]
fn dispel(player: EntityContext<PlayerKind>) {
    when(player.living().has_effect(EffectId::Speed)).then(mcfunction![
        player.living().damage(DamageAmount::hearts(0.5), DamageKind::Magic);
        player.living().clear_effect(EffectId::Speed);
        player.living().give_effect(EffectId::Glowing)
            .duration(Ticks::seconds(3)).particles(false);
    ]);
}
```

The maintained Trailforge grapple action uses the same builder to apply its
short speed and slow-falling buffs. Query and `EntityScope::bind` handles also
retain their targets through these operations. Mutations do not implicitly
change the executor, position, dimension, rotation, or anchor.

Damage amounts use `DamageAmount` (or a convertible fixed number), and effects
use canonical effect IDs and durations. Durations must be positive whole seconds;
`Ticks` inputs must be exactly divisible by 20. Invalid amounts, durations, or
selectors are reported by fallible pack export with the owning function. This
replaces the former immediate `CommandResult<String>` damage result. Cached
invalid actions remain invalid on every export. Explicit `build`/`try_build`
methods remain available when advanced command-layer code needs text.

These actions mutate **native Minecraft health and effects**. They do not
synchronize an unrelated Sand State field named `health`, take ownership from
archetype property bindings, install observers, or provide exact damage-event
payloads. Minecraft damage immunity, armor and other native rules still apply.
Native health reads remain `EntityData`; arithmetic uses `StatCurve`/`ScoreExpr`.
Generic entities and known nonliving kinds cannot obtain living capabilities.

This is partial coverage of #383/#381: fixed native damage and effect mutations
compose with existing reads and Conditions. General health writes, dynamic
damage/duration expressions, movement and exact damage observation are outside
this slice; both issues remain open.
