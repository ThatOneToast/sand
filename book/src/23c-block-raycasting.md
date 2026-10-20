# Block Raycasting

`BlockRaycast` registers a reusable synchronous scan along the current command
rotation. A cast preserves its executor and dimension; its callback runs at the
sample that ends the cast. Register the definition once and call `cast()` from
functions or event handlers without naming generated functions or counters.

```rust,no_run
use sand::command::Anchor;
use sand::prelude::*;
use sand::systems::raycast::BlockRaycast;

#[datapack_component]
fn mining_ray() -> BlockRaycast {
    BlockRaycast::new(ResourceLocation::new("demo", "mining").unwrap())
        .origin(Anchor::Eyes)
        .max_distance(5.0)
        .step(0.25)
        .scan_for(BlockPredicate::new().tag(TagId::minecraft("logs").unwrap()))
        .on_hit(found_log)
}

#[function]
fn found_log() {
    WorldContext::current().block(BlockPos::here()).replace(vanilla::Block::Air);
}

#[function]
fn use_tool() {
    mining_ray().cast();
}
```

`scan_for` skips nonmatching sampled blocks, including intervening walls.
`first_collision().pass_through(predicate)` stops at the first sampled block that
does **not** match the pass-through predicate. Supply that criterion explicitly;
Sand does not infer physical collision shapes from block names. Both modes use
the same `BlockPredicate` as location predicates and advancement conditions,
including block lists, block tags, state properties and block-entity data. Block
lists and tags are alternatives; specifying both is an export error.

Samples exclude the origin and occur at `step`, `2 * step`, and so on, ending at
exactly `max_distance`. A nonmultiple range shortens the final interval. A range
smaller than the step produces one sample at the range. Distance and step must
be finite and positive, and `ceil(range / step)` must not exceed 256. This is
discrete sampling: small features between samples can be skipped. It is not
voxel traversal or entity raycasting.

`Anchor::Eyes` applies the eye offset once before stepping; `Anchor::Feet` starts
at the supplied position. Rotation comes from the current command context,
normally the caster's view. An unloaded or out-of-world sample terminates as a
miss without loading chunks or claiming a collision. Exhausting the range also
terminates as a miss. Use `on_miss(callback)` when that needs a response; otherwise
the miss simply clears the counter and returns. An execution context without an
entity executor does nothing.

Callbacks accept registered function items or canonical `FunctionId`s naming
registered functions. Export resolves local namespaces and rejects unknown IDs
with the owning generated function and callback identity in the diagnostic.
Calls retain this validation even when `Actions` were cached before export. A
callback declared `context = player` runs only for a player caster. For a
nonplayer the cast still terminates at the hit or final miss sample, clears its
counter, and skips that callback; skipping a hit callback does not cause a second
miss. This guard applies to these retained registered calls, not arbitrary raw
function commands elsewhere in a datapack.

Each definition has an export-owned objective and each entity has its own
counter. Terminal callbacks run after the counter is cleared. A callback can
cast the same definition again without resuming the outer traversal, and nested
casts return to the callback's previous position and rotation. The sample limit
bounds each traversal, not arbitrary recursion authored inside user callbacks.

The maintained fixture is `sand/tests/block_raycast.rs`. Run its server proof
with `python3 scripts/validate_block_raycast.py --jar /path/to/26.2/server.jar
--java /path/to/java --output /tmp/raycast-proof`. It exports the actual Sand
fixture and checks two real players, nested casts, independent definitions,
clamped endpoints, tag and collision semantics, context guards, unloaded
samples, and server-context no-ops. The output directory retains server logs and
`evidence.json` with the passed runtime assertions.
