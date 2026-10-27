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
region observations or global-State conveniences.
Those should share the existing models rather than add independent polling or
storage systems.

## Furnace-family gameplay

`when_furnace` checks one address for a furnace, blast furnace, or smoker and
runs one grouped body. Its closure constructs the pack; it does not execute in
Minecraft. The same capability covers all three machines:

```rust,ignore
#[derive(State)]
#[state(namespace = "forge", scope = player)]
struct Inspection {
    elapsed: Score,
    total: Score,
    burn: Score,
}

#[function(context = player)]
fn inspect_machine() {
    let player = EntityContext::<PlayerKind>::default();
    let state = Inspection::on(player);
    WorldContext::current().block(BlockPos::here()).when_furnace(|furnace| {
        mcfunction![
            state.elapsed.set(furnace.cooking_elapsed());
            state.total.set(furnace.cooking_total());
            state.burn.set(furnace.burn_remaining());
            when(furnace.burn_remaining().gt(0)
                .and(furnace.input().matches(vanilla::Item::RawIron).unwrap()))
                .then_all([
                    WorldContext::current().block(BlockPos::above(1))
                        .replace(vanilla::Block::GoldBlock),
                ]);
        ]
    });
}
```

Call this from an explicit gameplay interaction or a registered `BlockRaycast`
hit callback, whose position is the hit sample. There is no automatic observer.
The same `StatCurve` reads also assign to global State. `input()`, `fuel()` and
`output()` return ordinary `ItemLocation`s for slots 0, 1 and 2. Use their existing
match/set/clear operations. A `set` replaces the stack; it is not an item transfer.
Vanilla item commands permit explicit output writes and do not enforce GUI
insertion restrictions or fuel/recipe validity.

The grouped body enters once even if it replaces a furnace with a smoker. A
return ends that body without ending its caller. Inline macro lines cannot be
moved into the generated helper: use an explicit macro function and
`FunctionMacroArgs::call_with`. Empty or discarded bodies emit no furnace work.

The entry guard proves the family only at entry. Replacing the block can make
later reads fail. A missing or unavailable read is not zero/idle: canonical
numeric failure skips that assignment's commit, retaining its old destination.
It does not roll back earlier actions or cancel later actions. Failed positive
comparisons are false, so negation alone cannot distinguish absence from idle.

Addresses do not capture a block or dimension. Relative/local coordinates resolve
where each operation executes; moving an action or value into another
`Target::each` or dimension body can rebind it. Keep reads and item locations in
the intended execution scope, or enter another guarded scope there.

The three reads expose native **serialized signed-short ticks**. Vanilla narrows
its internal integers to shorts, including wrapping values outside the short
range. Sand does not clamp negatives or reinterpret them as unsigned.
`cooking_elapsed()` can reset or cool down. `cooking_total()` is the stored total,
which may be zero or vanilla's 200-tick fallback when no recipe exists; it does
not prove that cooking can finish. `burn_remaining()` already accounts for
smoker/blast-furnace fuel scaling and does not imply productive cooking. These
are separate live reads, not an atomic multi-field snapshot.

This capability is verified against the 26.2 serialization contract. Exports
accept 26.2, normalized 26.2.0, and `latest` only while it resolves to 26.2.
Fallback and other target versions are diagnosed, even for cached or escaped
numeric reads. Unprofiled exports apply the same check to Sand's current baseline.
No claim is made for unverified versions.

The maintained authoring fixture is `sand/tests/furnace_gameplay.rs`; the dedicated
proof is `scripts/validate_furnace_gameplay.py --jar /path/to/26.2/server.jar
--output /tmp/furnace-proof`. It uses a disposable world and two real clients,
records source/JAR/pack identities, and checks real cooking, native reads,
State/Conditions, dimension/local context, failure after a valid read, grouped
returns, and reload. Running that proof is separate from exporting the fixture.
