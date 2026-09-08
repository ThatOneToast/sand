# Unified Data And Inventory

Sand models a data operation as one typed target plus one NBT path. The target
is storage, an entity, or a block entity; the same `NbtRef<T>` then supports
reads, writes, copies, collection edits, conditions, schema fields, and
inventory snapshots.

```rust,ignore
use sand::prelude::*;

let health = Nbt::entity(Target::self_()).path("Health");
let first_item = Nbt::block(BlockPos::here()).path("Items[0]");
let cache = Nbt::storage(ResourceLocation::new("trail", "cache").unwrap()).path("last_item");
```

`path(...)` creates an untyped reference for dynamic vanilla data.
`typed_path::<T>(...)` and schema fields retain an application value type.
This is useful Rust-side information, not a promise that Minecraft will
validate every runtime NBT shape.

## Paths and values

`NbtPath::new(...)` validates ordinary field, key, and list-index syntax when
the command is exported. Build paths structurally with `.field(...)`,
`.key(...)`, and `.index(...)` when convenient. `NbtPath::raw(...)` is the
explicit escape hatch for modded or newly added syntax: it renders unchanged,
cannot receive structural validation, and remains the author's responsibility.

Typed integers, floats, booleans, strings, lists, and `NbtCompound` values
render as SNBT. Raw SNBT remains explicit and opaque; Sand does not parse it.

```rust,ignore
let config = Nbt::storage(ResourceLocation::new("trail", "config").unwrap()).path("max_level");
config.set(10);
config.get();
config.get_scaled(10.0);
config.remove();

let queue = Nbt::storage(ResourceLocation::new("trail", "data").unwrap()).path("queue");
queue.append(1);
queue.prepend_from(&config);
queue.insert(2, NbtCompound::new().field("ready", true));
queue.merge(NbtCompound::new().field("owner", "trail"));
```

The typed data command IR represents `get`, `remove`, `merge`, and every
`modify` source/operation until the final renderer. Invalid resource
locations, empty or malformed paths, non-finite scales, invalid list indices,
multi-entity writes, and unsupported command/profile combinations become
structured build diagnostics instead of partial datapacks.

## Inventory locations

`ItemLocation` is the canonical live-item location model. Its entity and block
factories cover selected/main hand, offhand, armor, hotbar, main inventory,
ender chest, generic entity slots, and block-container slots.

```rust,ignore
let player = ItemLocation::entity(Target::self_());
let selected = player.mainhand();
let helmet = player.helmet();
let hotbar_three = player.hotbar(3)?;

let input = ItemLocation::block(BlockPos::here()).slot(0)?;
```

An `ItemLocation` is live. Its `.nbt()` view is an item-stack snapshot source
for `/data`; copying from one live item location to another uses `/item
replace`, which is the safe vanilla mutation family.

```rust,ignore
let selected = ItemLocation::entity(Target::self_()).mainhand();
let cache = Nbt::storage(ResourceLocation::new("trail", "cache").unwrap()).path("last_item");
selected.copy_to(&cache);

ItemLocation::block(BlockPos::here())
    .slot(0)?
    .copy_to(&Nbt::storage(ResourceLocation::new("trail", "cache").unwrap()).path("input"));

let offhand = ItemLocation::entity(Target::self_()).offhand();
offhand.replace_from(&selected)?;
```

Arbitrary player/entity inventory NBT writes are rejected: vanilla does not
safely expose them through `/data modify entity`. Block-container NBT writes
are supported. Use `.replace_from(...)` for live inventory mutation,
`.copy_to(...)` for snapshots, and `.copy_from(...)` only where the location
supports NBT writes.

`.matches(...)` and `.is_empty()` lower through typed `execute if items`
conditions; `.exists()` lowers through typed `execute if data`.

## Schema-owned field handles

Derived State is the single source of truth for scoped gameplay fields. The
derive creates lifecycle metadata and bound handles from one declaration:

```rust,ignore
#[derive(State)]
#[state(namespace = "trail", scope = player)]
struct PlayerData {
    #[state(default = 100)]
    mana: Score,
    #[state(default = false)]
    has_wand: Flag,
    #[state(auto_tick)]
    cast: Cooldown,
    #[state(default = "BossPhase::Idle")]
    phase: EntityEnum<BossPhase>,
}

let player = PlayerData::on(EntityContext::<PlayerKind>::default());
player.mana.gte(25);
player.has_wand.is_enabled();
player.cast.ready();
```

Schema provisioning applies defaults only to missing state, so reloads and
reconnects do not clobber values. Command storage remains pack-global; Sand
does not invent an implicit UUID-keyed record format.
