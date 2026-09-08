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
`EntityData` path. Player inventory changes should use typed `/item` operations,
not entity-NBT mutation. An `AnyEntity` context from `Target::entities()` exposes
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
owners of synchronization and lifecycle behavior; capability calls do not
create objectives, tags, scans, or reconciliation functions.

All of these APIs target Sand's supported Minecraft 26.x+ command model.
