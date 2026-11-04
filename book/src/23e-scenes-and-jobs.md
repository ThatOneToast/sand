# Scenes, spatial expressions and jobs

`Scene` and `Job` extend the same `Lifetime` used by abilities and effects.
Register each definition with `#[datapack_component]`, then call `start` or
`cancel` in owner context. Each instance has independent state and cleanup.
The [animated boss example](../../examples/animated_boss/) combines both systems.

## Animated displays

A scene contains up to 64 uniquely named block-display parts. Each part uses a
typed `BlockId` with its default block state and a local `Transform`:

```rust,ignore
use sand::prelude::*;
use sand::systems::{Lifetime, Scene, Transform};

#[datapack_component]
fn telegraph() -> Scene {
    Scene::new(Lifetime::new("demo:telegraph".parse().unwrap(), Ticks::new(60)))
        .part("core", BlockId::minecraft("amethyst_block").unwrap(), Transform::default())
        .keyframe("core", Ticks::new(1), Ticks::new(20),
            Transform::default().translated([0.0, 2.0, 0.0]).scaled([2.0; 3]).yaw(90.0))
        .follow_owner(true)
}
```

At loaded tick one, the display starts a twenty-tick interpolation toward the
specified pose. Minecraft clients interpolate the visual transform; Sand emits
commands at keyframe boundaries. Keyframes on the same part cannot overlap.
The animation runs once and holds its final pose until lifetime cleanup.
Translation, nonnegative scale and yaw must be finite. Scale is applied first,
then yaw, then translation. Positive yaw rotates local +X toward -Z around +Y;
it is independent of the entity's view rotation. Following tracks owner position,
not owner rotation.

All display entity positions remain at the root position; visual offsets belong
to the transformation. This keeps ordinary chunk loading aligned with the root.
Do not remove or manipulate Sand's private runtime entities. No chunks are forced
loaded, and unloads pause server-side animation time. Client interpolation can
finish while server work is paused; this is a visual animation, not a physics
clock. Displays have no gameplay collision. This first scene API is flat and
supports block displays, translation, scale and yaw; it does not yet provide
hierarchical bones, animation blending, item/text parts or animation loops.

## Bounded jobs

A job processes zero-based indices in ascending order. Its callback gets the
existing `TaskContext` and a `StatCurve` that reads the instance's current index.
Store authored progress on ordinary entity State attached by `Lifetime::owns`.

```rust,ignore
use sand::prelude::*;
use sand::systems::{Job, Lifetime};

#[datapack_component]
fn work() -> Job {
    Job::new(
        Lifetime::new("demo:work".parse().unwrap(), Ticks::new(100)),
        40,
        |ctx, _index| ctx.owner(|_| cmd::say("One work step")),
    )
    .per_tick(2)
    .on_complete(|ctx| ctx.owner(|_| cmd::say("Finished")))
}
```

The default budget is one step; valid budgets are 1–64. The budget applies to
each instance, so bound concurrency with `Lifetime::capacity`. This bounds
callback counts, not the cost of arbitrary commands inside a callback or MSPT.
Indices are expressions in **instance context**: evaluate or store an index
before traversing to an owner or another entity. The job does not snapshot
runtime expressions automatically across context changes.

Completion runs once after the last step, then stops the lifetime. Cancellation,
expiry, missing owners and reload invalidation discard unfinished work without
completion. Stopping inside a step also skips completion. The lifetime is a hard
loaded-tick deadline: allow enough duration for all batches to run. Jobs execute
after lifetime validation and its tick callback, independently of `Lifetime::every`.
Instances advance at most once per game tick, including when callbacks move them
between dimensions. Framework continuations and cleanup resolve the live marker
again after callback boundaries, because vanilla replaces non-player entities
during dimension transfer. Existing start/stop callbacks are preserved; an early return from cleanup cannot
skip framework cleanup. A callback that explicitly stops its instance stops
remaining work in that batch.

## Spatial expressions

`StatCurve::dot`, `cross`, `distance_squared` and `lerp` compose the canonical
checked numeric model. Three-dimensional vectors are arrays of three expressions;
there is no separate position or vector-state system to maintain.

```rust,ignore
let facing_score = StatCurve::dot(forward, direction_to_target);
let perpendicular = StatCurve::cross(forward, up);
let progress = StatCurve::lerp(start, end, weight);
```

Dot and cross do not normalize vectors. Interpolation does not clamp its weight;
values outside zero to one extrapolate. Existing fixed-point rounding and
intermediate overflow rules apply. On 26.3, supported checked additions can use
`compute`; multiplication retains the established integer lowering to preserve
rounding. Older targets use their compatible backend. These helpers do not claim
to cover arbitrary floating-point mathematics or all vanilla `compute` providers.
