# Hit volumes and action sequences

Use the canonical `Target` to choose candidates and `HitVolume` to describe
reusable area geometry. Use `Sequence` for finite, owned encounter timelines.
Neither introduces a new gameplay state model: authored progress remains State,
and sequence ownership, admission and cleanup remain `Lifetime` responsibilities.

## Area hits

```rust,ignore
use sand::prelude::*;
use sand::entity::HitVolume;

let area = HitVolume::sphere(4.0).unwrap();
let hits = Target::players()
    .within_blocks(8.0)
    .sort(sand::command::SortOrder::Nearest)
    .take(16).unwrap()
    .each_in(area, |player| {
        player.living().damage(4.0, sand::command::DamageKind::Magic)
    });
```

The callback executes as and at each hit, with the usual typed `EntityContext`.
A sphere requires a positive radius and includes entity origins at its radius; it does not intersect their
bounding boxes. `HitVolume::cuboid([width, height, depth])` instead intersects
native entity hitboxes. It is centered on the command position and aligned to
world axes. Full side lengths must each be at least one block, matching the
minimum extent of native selector volumes. Touching only a box boundary follows
vanilla's strict intersection rules.

`volume.translated([x, y, z])` sets an offset from the current command position,
using world axes. It replaces any previous offset. A common box around a standing
entity is `HitVolume::cuboid([4.0, 2.0, 4.0])?.translated([0.0, 1.0, 0.0])?`.

**Candidate limits apply before geometry.** Existing Target filters, sorting and
limits select candidates around the original command position. The translated
volume then filters those candidates. A limit of one can therefore produce zero
hits even if another entity lies inside the volume. This is an explicit candidate
budget, not a promise to return the nearest successful hit. Broaden the candidate
range appropriately when translating the volume.

Queries use native selector checks without helper entities, tags or scoreboard
scratch state. Nested queries preserve the outer callback context. They do not
exclude the caster, check line of sight, or deal damage automatically. These are
instantaneous queries, not swept collision; cones, rotated boxes and sub-block
boxes are not supported by this first volume API. Existing `BlockRaycast` remains
the separate block-sampling facility.

## Timed actions

```rust,ignore
use sand::prelude::*;
use sand::systems::{Lifetime, Sequence};

#[datapack_component]
fn attack() -> Sequence {
    Sequence::new(Lifetime::new("demo:attack".parse().unwrap(), Ticks::new(61)))
        .then(|ctx| ctx.owner(|_| cmd::say("Charging")))
        .wait(Ticks::new(40))
        .then(|ctx| ctx.owner(|_| cmd::say("Strike")))
        .wait(Ticks::new(20))
        .on_complete(|ctx| ctx.owner(|_| cmd::say("Ready")))
}
```

`start()` admits an independent instance. The first action runs on its first
loaded tick, not synchronously during start. In this example, charging happens
on loaded tick 1, striking on tick 41, and completion on tick 61. Adjacent actions
run on the same tick in insertion order; a zero-length wait preserves that order.
A trailing wait delays completion. The lifetime must allow at least the sum of
waits plus one loaded tick for normal completion.

Sequences compile through the existing indexed Job runtime, with one cursor
advance per loaded tick regardless of the lifetime's user callback cadence.
Definitions accept 1–256 actions and total waits up to 2,147,483,646 ticks. Each
action adds a tick dispatch condition; the action body runs only when due. This
bounds authored timeline size, not arbitrary command cost or MSPT.

Each action is isolated in a function. A Minecraft early return ends that action,
while `ctx.stop()` explicitly cancels the entire sequence, including later actions
on the same tick. Continuations resolve the live instance after dimension changes.
`cancel()` cancels this definition's instances belonging to the current owner.
Cancellation, owner invalidation and expiry discard unfinished actions and skip
completion, while preserving Lifetime cleanup callbacks.

Use ordinary `when` conditions inside actions for conditional gameplay and
State transitions for persistent behavior. For concurrent child work, bounded
condition waits and joins, see [task composition](23g-task-composition.md).
`ctx.start_child(system)` makes the relationship explicit and gives the parent
instance responsibility for cleanup. Ordinary starts remain independent.
