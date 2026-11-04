# Scenes, spatial expressions and jobs

`Scene` and `Job` extend the same `Lifetime` used by abilities and effects.
Register each definition with `#[datapack_component]`, then call `start` or
`cancel` in owner context. Each instance has independent state and cleanup.
The [animated boss example](../../examples/animated_boss/) combines both systems.

## Animated displays

A scene contains up to 64 uniquely named block, item and text-display parts.
Block parts use a typed `BlockId` with its default state; `.item` accepts the
canonical component-bearing `ItemStack`, and `.text` accepts `TextComponent`.
All part kinds share local `Transform`, keyframes and lifetime cleanup:

```rust,ignore
use sand::prelude::*;
use sand::systems::{Lifetime, Scene, Transform};

#[datapack_component]
fn telegraph() -> Scene {
    Scene::new(Lifetime::new("demo:telegraph".parse().unwrap(), Ticks::new(60)))
        .part("core", BlockId::minecraft("amethyst_block").unwrap(), Transform::default())
        .item("icon", ItemStack::new(ItemId::minecraft("diamond").unwrap()),
            Transform::default().translated([0.0, 1.0, 0.0]))
        .text("label", Text::new("Incoming").red(),
            Transform::default().translated([0.0, 2.0, 0.0]))
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

`local.then(parent)?` composes a reusable local pose under an outer pose,
including scaling and rotating its translation. Uniform parent scale always
composes. Nonuniform scale and yaw can introduce shear, which the yaw-only
model rejects with a diagnostic. This computes one flat pose before export;
it does not introduce a second hierarchy or a server-side transform system.

Items retain their counts and typed components and use Minecraft's default
`none` display context. They cannot be picked up or transferred as inventory.
Text uses native fixed-billboard, alignment, background and line-width defaults.
Dynamic text resolves at creation; keyframes animate geometry, not text content.

All display entity positions remain at the root position; visual offsets belong
to the transformation. This keeps ordinary chunk loading aligned with the root.
Do not remove or manipulate Sand's private runtime entities. No chunks are forced
loaded, and unloads pause server-side animation time. Client interpolation can
finish while server work is paused; this is a visual animation, not a physics
clock. Displays have no gameplay collision. Scenes are flat and support
translation, scale and yaw; they do not provide hierarchical bones, animation
blending, arbitrary shear or animation loops.

## Owned screen presentation

`Hud` composes the same player `Lifetime` with native actionbar and bossbar
presentation. Text and bossbar values evaluate in owner context each loaded
tick. Bossbar values accept canonical State accessors and `StatCurve`, round to
integer units and clamp to zero through the positive maximum. Arithmetic
failure preserves the previous native value. Declare required player State
with `Lifetime::requires` as for other owned gameplay.

```rust,ignore
use sand::prelude::*;
use sand::systems::{Hud, Lifetime};

#[datapack_component]
fn status_hud() -> Hud {
    Hud::new(Lifetime::new("demo:status_hud".parse().unwrap(), Ticks::seconds(10))
        .capacity(16))
        .actionbar(Text::new("Ready"))
        .bossbar(Text::new("Progress"), 25, 100)
}
```

Call `start` and `cancel` in player context. Each instance owns a distinct
bossbar shown only to its player. Capacity is at most 128 per definition;
cleanup removes both the bar and its compact live-list entry. Reload removes
all recorded bars, including bars whose roots are unloaded or owners offline.
Owner loss, expiry and cancellation use ordinary lifetime cleanup. Removing
the datapack itself cannot invoke cleanup.

Minecraft has one actionbar slot per player. The newest started Sand HUD in
the export pack claims that slot, including definitions imported from other
resource namespaces. Pack-wide claim identities never reuse or wrap; exhaustion
rejects new actionbar HUD starts. Superseded instances stop writing and
cannot clear newer presentation. Ending the newest HUD clears its lease; older
HUDs do not resume. Other datapacks can overwrite the same native slot. Native
bossbar color/style defaults apply; this API does not claim to multiplex the
actionbar or provide a general screen layout engine.

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
