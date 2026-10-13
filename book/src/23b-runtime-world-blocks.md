# Runtime world and block gameplay

`WorldContext::current()` describes the dimension and position where a generated
function runs. It is separate from `build::World`, which configures a generated
world. Runtime handles do not load chunks, scan regions, or store a persistent
world identity.

Use a block handle to share one typed position across checks and actions:

```rust,ignore
let world = WorldContext::current();
let chest = world.block(BlockPos::absolute(10, 64, 20));
let reward = ItemStack::new(vanilla::Item::Diamond.into());
let slot = chest.inventory().slot(0).unwrap();

when(chest.matches(vanilla::Block::Chest))
    .then_all([slot.set(reward).unwrap()]);
```

`matches` returns the canonical `Condition`. `replace` and `destroy` return owned
`Actions`, preserving the typed block state and position until export validates
them. `BlockState` can specify state properties; a typed vanilla block converts
to the same canonical model. Conditions compose with State, living-entity, and
inventory checks.

`destroy()` leaves air and requests vanilla destruction drops. `replace(...)`
uses vanilla replacement behavior. Neither operation promises atomic inventory
transfer or preservation of a previous block entity's contents.

## Dimension and position scope

Scope a complete body when a mechanic belongs to another dimension:

```rust,ignore
let nether: sand::registry::DimensionId = "minecraft:the_nether".parse().unwrap();
let actions = WorldContext::current().in_dimension(nether, |world| {
    let block = world.block(BlockPos::absolute(1, 64, 2));
    when(block.matches(vanilla::Block::Stone))
        .then_all([block.replace(vanilla::Block::GoldBlock)])
});
```

The condition, mutation, inventory addressing, and data access in that body use
the selected execution context. Minecraft scales the execution origin across
dimensions with different coordinate scales. Absolute block positions stay
absolute; relative positions use the resulting origin. The executor remains
the caller. A nested entity query may establish its own position and dimension.
After the body, surrounding actions retain their caller's original context.

A handle does not capture the dimension or resolve relative coordinates at Rust
construction time. Keep its operations in the intended generated execution
scope, just as with `EntityContext`.

## Players and entities in the world context

`world.players()` and `world.entities()` return the canonical `Target`. They
select only the dimension where their generated actions run, including when
used inside `in_dimension`. Unlike a bare `Target::players()`, the world player
query cannot accidentally include players from other dimensions.

```rust,ignore
WorldContext::current().in_dimension(nether, |world| {
    world.entities().tag("arena_enemy").within_blocks(32.0).each(|_| {
        world.block(BlockPos::here()).replace(vanilla::Block::GoldBlock)
    })
});
```

Use the normal target filters, narrowing, and `each` execution API. Construction
does nothing at runtime; iteration is explicit. There is no implicit tick scan
or maximum radius. `within_blocks` bounds work around the current origin, and
`each` then runs at each selected entity's own position. Queries retain the
execution context at use, not the dimension where the Rust value was built.

## Loaded blocks and specialized capabilities

A block address does not prove that its chunk is loaded, that the block is a
container, or that it has block-entity data. `inventory()` returns the existing
`BlockInventory` address builder and validates slot indices; the live block
must still support the requested slot. `data()` is advanced access to the
canonical `Nbt` root, with typed paths and values. It does not create a second
block data model or make unsupported block entities writable.

A failed or unloaded block query is false; negation therefore also matches an
unloaded block. Mutations and data/inventory operations follow vanilla failure
behavior. Sand generates no implicit chunk-loading or tick work.

This runtime slice does not yet provide time/weather controls, typed gamerules,
specialized furnace slots, region observations, or global-State conveniences.
Those should share the existing models rather than add independent polling or
storage systems.
