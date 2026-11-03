# Child tasks, condition waits and joins

`TaskContext::start_child` starts registered work for the original owner while
binding its lifetime to the **current parent instance**. It supports `Lifetime`,
`Job`, `Sequence`, `Scene`, `Projectile` and `Ability` through their canonical
Lifetime reference. Normal admission guards, ability costs and capacity remain
in effect. A rejected start creates no child and does not hold a join open.

```rust,ignore
use sand::prelude::*;
use sand::systems::{Job, Lifetime, Sequence};

#[datapack_component]
fn work() -> Job {
    Job::new(
        Lifetime::new("demo:work".parse().unwrap(), Ticks::new(40)),
        20,
        |ctx, _| ctx.owner(|_| cmd::say("Working")),
    )
}

#[datapack_component]
fn encounter() -> Sequence {
    Sequence::new(Lifetime::new("demo:encounter".parse().unwrap(), Ticks::new(120)))
        .then(|ctx| {
            let mut tasks = ctx.start_child(work());
            tasks.extend([ctx.start_child(work())]);
            tasks
        })
        .wait_until(|ctx| ctx.children_finished(), Ticks::new(100))
        .then(|ctx| ctx.owner(|_| cmd::say("Both children stopped")))
}
```

Both jobs can run concurrently. They are independent instances, even though they
share a definition and an owner. Stopping one parent cancels its descendants,
without cancelling another parent's children belonging to that same owner.
Starting another system through `ctx.owner(|_| system.start())` remains detached
work: it does not acquire the parent relationship.

## Cancellation and unloaded work

Loaded descendants in the parent's configured dimensions stop when the parent
stops. Other descendants validate cancellation on their next loaded tick. A
bounded ancestry ledger preserves invalidation even through unloaded intermediate
parents and owner-wide cancellation of an unloaded parent. No chunks are loaded.
Reload invalidates old ancestry. Nesting is capped at 32 generations; a deeper
start is rejected. A stopping parent cannot start replacement children from its
cleanup callback.

State still belongs in ordinary State declarations. The runtime owns ancestor
identities, live-child counts and cleanup bookkeeping. Do not mutate its private
entities or storage. Shared refreshable status effects are not child tasks:
refreshing an existing owner stack would otherwise transfer someone else's work
into a new parent.

## Conditions and deadlines

`Sequence::wait_until(predicate, timeout)` begins checking on the loaded tick
after preceding actions. If already true, later actions run on that tick. If
false, the timeline cursor pauses; subsequent waits are relative to release.
After `timeout` failed checks, the sequence stops and skips completion. The
Lifetime's hard deadline continues counting while blocked. Timeouts must be
positive, and definitions support at most 256 condition gates.

The predicate runs in instance context. Use `ctx.owner_matches(|owner| ...)` to
test typed owner State without losing the instance context. Missing owners make
this condition false. The [boss example](../../examples/animated_boss/) waits for
its attack job to update the owner's phase, with a timeout, instead of assuming a
fixed completion tick.

`children_finished()` joins direct children and means **all have stopped**, not
that all completed successfully. Cancellation and expiry also settle a child.
No children is true for a live parent; a stopped parent is false. An unloaded
child remains outstanding until its cleanup runs. Use a timeout for joins and
store explicit outcome State when gameplay needs success/failure distinctions.

This provides concurrent child work, bounded condition waits and joins. Named
race groups, task return values and general behavior-tree selection remain
separate future extensions.

## First successful hits and projectile precision

`Target::take(n)` bounds candidate count while retaining a many-target type.
`limit(1)` remains the static single-target narrowing operation.

`first_in(volume, callback)` checks candidates in Target order and runs at most
one callback for the first geometry match. Use `.sort(SortOrder::Nearest)` and
`.take(n)` to find the nearest successful hit among a bounded candidate set.
Using `.nearest()` would restrict the search to one candidate before geometry.
A callback that returns zero still counts as a hit; its return is isolated from
the caller. Queries do not implicitly test line of sight.

Projectile samples are no farther apart than `min(radius, 0.25)` blocks. Export
rejects configurations needing more than 256 samples per tick; reduce speed or
increase radius to fit that budget. Native tests cover a small-radius target
between the old quarter-block samples. This remains discrete point sampling:
there is no claim of exact swept hitboxes, voxel traversal, or tangent-contact
coverage. Exact swept collision and cone/rotated-volume queries remain future
work.
