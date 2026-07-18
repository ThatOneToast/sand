# Item locations and event-time item snapshots

This page documents `sand_core::item` (#229 Phase 7): typed item
*locations* and immutable, storage-backed event-time item *snapshots*. It
does not cover participant-rich event contexts (attacker/victim,
interacted-entity recovery, projectile/ammunition correlation) — that is
#230, deliberately out of scope here. See `ai/known-limitations.md`
`LIM-ITEM-001`..`LIM-ITEM-003` for what remains unverified or deferred.

## Root problem

A `SandEvent` handler cannot safely identify "the item involved in this
event" by reading a player's/entity's/container's current inventory state
once the handler runs. By handler time, vanilla may already have consumed,
moved, damaged, or otherwise transformed the item (e.g. `consume_item`
decrementing a stack, a thrown item leaving the inventory entirely). Sand
needs a way to capture item data *before* that happens, and to hand
handlers immutable, already-captured data rather than a live re-read.

`sand_core::item` splits this into two typed pieces:

- [`ItemLocation`] — *where* an item lives right now: a hand, an equipment
  slot, a validated inventory/hotbar index, a block container slot, an item
  entity's stack. Never a raw slot string.
- [`ItemSnapshot`] — *what was captured* from a location at a specific
  generated-command point: immutable, storage-backed data with an explicit
  reliability level and a bounded lifetime. Never a live re-read.

## `ItemLocation`

```rust
use sand_core::item::{ItemLocation, HotbarIndex, InventoryIndex};
use sand_core::EquipmentSlot;

ItemLocation::PlayerMainHand;
ItemLocation::PlayerOffHand;
ItemLocation::PlayerEquipment(EquipmentSlot::Chest);         // fallible ctor rejects Mainhand/Offhand/Body
ItemLocation::PlayerHotbar(HotbarIndex::new(3)?);             // 0..=8, validated
ItemLocation::PlayerInventory(InventoryIndex::new(20)?);      // 0..=35, validated
ItemLocation::entity_equipment(some_selector, EquipmentSlot::Head)?;
ItemLocation::BlockContainer { position, slot: ContainerIndex::new(10)? };
ItemLocation::ItemEntity(some_selector);
```

Each variant's `.nbt_source()` renders the exact vanilla NBT source
(`DataTarget` + path) it addresses:

| Location | Renders as |
|---|---|
| `PlayerMainHand` | `entity @s SelectedItem` |
| `PlayerOffHand` | `entity @s Inventory[{Slot:-106b}]` |
| `PlayerEquipment(Chest)` | `entity @s Inventory[{Slot:102b}]` (armor slots use the canonical 100-103 inventory addressing, not `ArmorItems`) |
| `PlayerHotbar(n)` / `PlayerInventory(n)` | `entity @s Inventory[{Slot:<n>b}]` — hotbar and inventory share one canonical addressing space (0-35) |
| `EntityEquipment { entity, slot }` | `entity <selector> ArmorItems[n]` / `HandItems[n]` (non-player living entities) |
| `BlockContainer { position, slot }` | `block <pos> Items[{Slot:<n>b}]` |
| `ItemEntity(selector)` | `entity <selector> Item` |

`PlayerEquipment`/`entity_equipment` reject `EquipmentSlot::Body` (no
stable single-item backing tag verified across supported versions — see
`LIM-ITEM-002`) and reject `Mainhand`/`Offhand` for `PlayerEquipment`
specifically (use the dedicated `PlayerMainHand`/`PlayerOffHand` variants
instead, which is why raw `EquipmentSlot` alone is never sufficient).
Index constructors return `ItemLocationError::IndexOutOfRange` rather than
panicking or silently clamping.

`ItemLocation` rendering does not depend on the active `VersionProfile` —
the NBT shapes above are stable across every version Sand supports
(1.18-26.2). If a future version changes slot addressing, that becomes a
new, explicitly version-gated code path rather than a silent behavior
change.

## `ItemSnapshot`

An `ItemSnapshot` never holds live item data in the Rust process — Sand is
a compiler, not a runtime agent on the server. It describes *generated
storage* (a `SnapshotSchema { storage, key }`) and typed accessors into it:

```rust
use sand_core::item::{ItemSnapshot, SnapshotSchema, SnapshotReliability};

let schema = SnapshotSchema::new("mypack:snapshots", std::any::type_name::<MyEvent>());
let (snapshot, commands) = ItemSnapshot::capture(
    &ItemLocation::PlayerMainHand,
    schema,
    SnapshotReliability::Exact,
)?;
```

`capture` always succeeds for a valid `ItemLocation` (capture itself is
never "may fail based on live state") and returns exactly four commands:

```mcfunction
data modify storage mypack:snapshots snap.<key>.present set value 0b
data modify storage mypack:snapshots snap.<key>.item set value {}
execute if data entity @s SelectedItem run data modify storage mypack:snapshots snap.<key>.item set from entity @s SelectedItem
execute if data entity @s SelectedItem run data modify storage mypack:snapshots snap.<key>.present set value 1b
```

Reset-then-conditionally-copy: `present` is unconditionally cleared first,
so a location that turns out to be empty produces a clean, explicit
"absent" snapshot rather than stale data from a previous capture at the
same schema.

### Absence, not `minecraft:air`

Item absence is never encoded as an item ID. `.is_present()` /
`.is_absent()` return a typed `Condition` (`StorageExists { .. }` testing
`snap.<key>{present:1b}`) that the caller embeds in generated `execute
if/unless data storage ...`. There is no `Option<ItemSnapshot>` on the Rust
side — presence is a runtime fact checked at generation time, not
something Sand's compiler can know at export time.

### Typed field access

```rust
snapshot.item_path()       // snap.<key>.item
snapshot.id_path()         // snap.<key>.item.id
snapshot.count_path()      // snap.<key>.item.count
snapshot.components_path() // snap.<key>.item.components
```

Each returns a `NbtPath`, so callers compose with the existing
`sand_core::state::storage` and `sand_commands::nbt` APIs rather than
building raw path strings.

### Reliability

| Level | Meaning |
|---|---|
| `Exact` | Copied from the authoritative event-time source before any Sand-generated mutation — used for tick-backed captures in `EventSetup::pre_observation`, which genuinely runs before Sand's own condition test. |
| `ExactPostTrigger` | Exact *at the moment of capture*, but vanilla may already have transformed the item before the trigger handed Sand control (e.g. some advancement criteria fire after the triggering action completes). Honest acknowledgment, not a downgrade to guesswork. |
| `Correlated` | Bounded observation, not directly supplied by the event source. Not produced by anything in Phase 7; reserved for future correlation work. |
| `Unavailable` | No capture was possible. Not produced by `capture()` itself (which always succeeds for a valid location); reserved for callers building higher-level APIs on top. |

Phase 7 only emits `Exact`/`ExactPostTrigger`, chosen explicitly by the
caller based on which lifecycle hook the capture commands are embedded
into — `capture()` does not infer or default this, so authors can't
accidentally claim a stronger guarantee than their integration provides.

### Capture ordering contract

- Tick-backed `SandEvent`s: embed the capture commands as (part of)
  `EventSetup::pre_observation`. This genuinely runs before Sand's own
  condition test each tick, so `Exact` is honest.
- Advancement-backed `SandEvent`s: embed the capture commands as the first
  lines of the handler body. This is the earliest point Sand controls, but
  vanilla's own criterion-firing order relative to inventory mutation is
  criterion-specific and not verified here — use `ExactPostTrigger`.
- **Not supported in Phase 7**: capturing at a Phase 6 advancement-backed
  graph *bridge parent*. Bridge parents have no `EventSetup`/handler body
  seam of their own (see `sandevent-advancement-graph-parent` in
  `ai/capability-manifest.yaml`) — there is nowhere to embed capture
  commands ahead of the bridge's synchronous dispatch. This is deferred to
  #230's graph-integrated context work, not silently worked around.

`sand-core/tests/item_snapshot_tick_capture_export.rs` proves this
ordering end-to-end through the real export pipeline for the tick-backed
case: the capture commands are the first lines of the generated check
function, strictly before the condition test, which is strictly before
`post_observation` cleanup.

### Storage, collisions, and concurrency

`SnapshotSchema::new(storage, event_label)` derives a short deterministic
key from `event_label` via the same FNV-1a scheme as event graph resource
keys (`tick_event_resource_key`), so two distinct `SandEvent` types never
collide, and the same type always regenerates the same path (deterministic
double-generation).

There is intentionally **no per-player key**. Command storage is global,
not vanilla-namespaced per player (`LIM-VAN-001`) — a per-player-keyed path
would require knowing the player ahead of time, which is impossible to
generate statically at export time. Cross-player safety instead relies on
Minecraft's own execution model: `execute as @a ... run function X` runs
each player's full synchronous call tree to completion before moving to
the next player (single-threaded, non-interleaved). A snapshot's
documented lifetime — *valid for the capturing invocation and its
synchronous descendant calls, then implicitly overwritten by the next
capture* — never outlives one player's synchronous turn, so this is safe
without indirection. If a future need requires a snapshot to survive past
one player's synchronous turn (e.g. into the next tick), that is persisted
correlation state and needs new, explicit design — not something Phase 7
provides or silently approximates.

### Cleanup

`.cleanup_commands()` resets `present` to `0b` and `item` to `{}` —
identical shape to the reset half of `capture()`. Nothing is cleaned up
automatically; callers wire cleanup into `EventSetup::post_observation`
(or their own handler tail) if they want deterministic reset between
invocations rather than relying on the next `capture()` call's own reset.

## Integration seam for future participant-rich contexts (#230)

```rust
pub struct EventItem {
    pub role: ItemRole,
    pub snapshot: ItemSnapshot,
}

pub enum ItemRole {
    UsedItem, Weapon, Tool, ProjectileItem, Ammunition, DroppedItem, EquippedItem,
}
```

This is the smallest stable seam #230 needs to attach a role to a
snapshot inside a richer event context. Phase 7 does not construct
`EventItem` anywhere itself and does not add participant types (attacker,
victim, interacted entity) — only the item-role vocabulary, since it's
already fully specified by this phase's own data model.

## What Phase 7 does not do

- Does not auto-wire capture into the `#[event]` macro or the tick
  coordinator's generated pipeline. A `SandEvent` author calls
  `ItemSnapshot::capture()` themselves and embeds the returned commands
  into their own `EventSetup`/handler body, as shown above and in
  `sand-core/tests/item_snapshot_tick_capture_export.rs`.
- Does not support capture at advancement-backed graph bridge parents
  (see above).
- Does not implement attacker/victim, interacted-entity, or
  projectile/ammunition correlation — that's #230.
- Does not convert an `ItemMatcher` (a predicate) into an `ItemSnapshot`
  (captured data), or vice versa — they answer different questions and
  Phase 7 keeps them separate.

[`ItemLocation`]: ../sand-core/src/item/location.rs
[`ItemSnapshot`]: ../sand-core/src/item/snapshot.rs
