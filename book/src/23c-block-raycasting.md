# Block Raycasting

`BlockRaycast` registers a reusable synchronous scan along the current command
rotation. A cast preserves its executor and dimension; its callback runs at the
sample that ends the cast. Register the definition once and call `cast()` from
functions or event handlers without naming generated functions or counters.

Use `ResourceLocation::local("mining")` when a reusable definition should belong
to the consuming pack's export namespace. Use `ResourceLocation::new` when the
resource deliberately has a fixed namespace.

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

#[on_event]
fn on_block_placed(_event: Event<sand::events::BlockPlaceEvent>) {
    mining_ray().cast();
}
```

The block-placement event runs as the placing player, so the same definition
casts from that player's eyes and view. This example is maintained in the
`pass_block_raycast` compile fixture.

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
be finite and positive, and `ceil(range / step)` must not exceed 256. Before
taking the ceiling, a quotient within two floating-point representable values
of a positive integer is rounded to that integer. This avoids a duplicate
endpoint from decimal rounding, such as `0.9 / 0.03` producing slightly more
than 30. The final interval still clamps to the supplied range; genuine
fractional remainders outside that tolerance retain an additional sample. This is
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

For generated-work accounting, count executed `.mcfunction` lines across all
private helper functions, including unsuccessful guards and helper-call lines.
This is an exact emitted-path count, not vanilla debug evaluations or MSPT.
A function containing only `ray.cast()` executes one call line and three entry
lines. Each continuing sample executes three advance lines and six sample
lines. The final range-exhaustion sample executes two advance lines, five sample
lines, and two completion lines (counter reset plus callback call or return).
Thus a loaded full-range miss with `N = ceil(range / step)` executes `4 + 9*N`
lines before callback work: **220** for the default range 6 / step 0.25 (24
samples), or **2,308** at the 256-sample cap. These paths invoke `2*N + 2`
generated helper functions, included in those line counts, and perform `N + 2`
counter writes. A supplied applicable callback adds one function invocation and
its own body work. A hit at sample `j < N` costs `4 + 9*j` lines before callback
work; a hit at the final sample costs `3 + 9*N`. Unloaded/out-of-world misses
terminate earlier. A server-context no-op executes only the caller line and the
entry guard. Predicate evaluation and nested command dispatch have costs not
represented separately by this line metric; event dispatch, callback bodies and any casts they
start are additional work, with no whole-chain or user-recursion bound.

The maintained fixture is `sand/tests/block_raycast.rs`. Run its server proof
with `python3 scripts/validate_block_raycast.py --jar /path/to/26.2/server.jar
--java /path/to/java --output /tmp/raycast-proof`. It exports the actual Sand
fixture and checks two real players, nested casts, independent definitions,
clamped endpoints, tag and collision semantics, context guards, unloaded
samples, and server-context no-ops. The output directory retains server logs and
`evidence.json` with the runtime assertions and final status. Evidence records
the tested Git commit/tree and worktree status, fixture/harness hashes, a
per-file generated-pack manifest and aggregate SHA-256, and the server JAR
SHA-256. The 26.2 vanilla CI job runs this harness and uploads its output on
failure.

A concise recorded proof is retained in
`tests/evidence/block-raycast-26.2.json`; rerunning the harness produces the full
per-file pack manifest and assertion conditions for comparison.
