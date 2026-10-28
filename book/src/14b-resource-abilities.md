# Shared Resources And Abilities

`sand::systems::ability` provides three small compositions. `ResourcePool` binds
an integer State score and a capacity `StatCurve`. `Ability` combines that pool
with a bound Cooldown, cost, duration and eligibility Condition.
`ResourceMaintenance` installs regeneration separately, once for a shared pool.
They use the same public State, Conditions, Actions and registration APIs as
ordinary datapack code. No gameplay constants or hidden player data are supplied.

Declare defaults and bounds on your own State schema, and mark the cooldown and
regeneration Timer `auto_tick`. Bind those fields to the event's player or an
injected `EntityContext<PlayerKind>`. The pool capacity expression must agree
with the State upper bound: State remains the authority for saturating writes.
Costs are nonnegative integers. Cooldown durations and regeneration cadence
must be positive and no greater than `i32::MAX` ticks; regeneration is positive.
Invalid configuration returns a diagnostic before registration.

Trailforge uses a 100-point stamina pool, boots, costs of 30 and 10, and different
raycast targets. The independent `examples/aether_tools` project depends only on
`sand`. It uses a 48-point Battery, an amethyst prism in the offhand, costs of 7
and 11, a 12-tick cooldown and regeneration of 3 every 7 ticks. Its reserve
ability binds different resource/cooldown fields and runs independently.

```rust,ignore
use sand::systems::ability::{Ability, ResourcePool};

let state = Battery::on(player);
let policy = Ability::new(
    ResourcePool::new(state.charge, 48),
    state.pulse,
    7,
    Ticks::new(12),
    player.inventory().offhand().matches(prism().item_predicate()?)?,
)?;
policy.accept(
    WorldContext::current().block(BlockPos::below(1))
        .matches(vanilla::Block::DiamondBlock),
    state.activations.add(1),
);
```

Change the item or slot by changing the existing `ItemLocation::matches`
condition. Combine it with unlock, region or other gameplay Conditions. Checks
read current equipment at activation; there is no equipment watcher. Use
`ready()` for UI and `attempt(search_actions)` for a synchronous target search.
The search should call `accept(target_condition, effects)` at one terminal
accepted target; BlockRaycast's hit callback preserves the caster for this use.
An immediate target can call `accept` directly.

Acceptance rechecks the whole eligibility condition together with the target,
then runs one grouped branch: subtract cost, start cooldown, issue effects.
Exactly-cost activation still completes cooldown and effects after the balance
becomes zero. Rejection, a miss, missing equipment and an active cooldown leave
both fields unchanged. A second activation sharing the cooldown is rejected.
Supply owned Actions or `cmd::function(typed_callback)` as effects.

Acceptance does not prove every native operation succeeds. A failed block write
or an early return in the effect callback cannot undo payment; early returns can
skip later callback actions. Do not expose an effect-only continuation as a safe
activation entrypoint. Asynchronous targets need their own ownership/lifecycle
protocol; this composition makes no reservation or rollback promise.

Register maintenance separately with `#[datapack_component]`. Return a
`ResourceMaintenance::new(unique_id, Target::players().each(...))` whose per-player
body calls `pool.regenerate(state.recharge, 3, Ticks::new(7))`. The same pool used
by two abilities has exactly one maintenance registration. State auto-ticking
runs before maintenance; a full pool skips the write but restarts the timer.
Repeated registration of the same function identity fails export with a resource
collision. Different identities are independent; giving the same pool two
identities is an authoring error and schedules two actions.

Construction is inert. Importing, constructing and discarding a pool, ability,
maintenance definition or Actions adds no resources beyond the same State
baseline. Registering maintenance adds a function and tick call even if no ability
is ever activated. The schema's own lifecycle is separate from feature usage.
State persistence, reconnect initialization and reload behavior remain canonical.

Discover the API with `sand api module sand::systems::ability`,
`sand api show sand::systems::ability::Ability::accept`, or
`sand api search ResourceMaintenance`.
