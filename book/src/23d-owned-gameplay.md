# Owned gameplay: abilities, projectiles and statuses

Sand definitions compile gameplay into vanilla datapack resources. A definition
is registered once with `#[datapack_component]`; its action methods can be called
from functions, events and systems. Rust callbacks build actions at export time.
Minecraft executes those actions later.

The complete [owned combat example](../../examples/owned_combat/README.md) lets
players sneak to cast a fireball. It uses the same player State for mana,
cooldowns, regeneration and diagnostics.

```rust,ignore
#[datapack_component]
fn fireball() -> Ability {
    Ability::projectile(
        Projectile::new(
            Lifetime::new("game:fireball".parse().unwrap(), Ticks::seconds(2))
                .origin(sand::command::Anchor::Eyes)
                .capacity(64)
                .requires::<Mage>(),
        )
        .speed(1.5)
        .radius(1.75)
        .targets(Target::players())
        .on_hit(|hit| hit.damage_target(6.0, DamageKind::Magic)),
    )
    .cost(Mage::mana, 20)
    .cooldown(Mage::fireball_cooldown, Ticks::seconds(3))
}

#[on_event]
fn cast_on_sneak(_: Event<sand::events::PlayerStartSneakingEvent>) {
    fireball().activate();
}
```

Import definitions from `sand::systems` and normal authoring types from
`sand::prelude`. Mark the cooldown field `#[state(auto_tick)]`. Admission checks
cost, cooldown, custom conditions, membership and capacity before creating work.
An admitted instance commits cost and cooldown before State attachment hooks or
user callbacks. `Ability::projectile` shares the projectile's admission boundary,
so a full projectile pool cannot spend mana. `ability.diagnose()` reports failed
requirements and active capacity to the current player without activating it.

## Ownership and cleanup

`Lifetime<K>` represents work owned by an entity of kind `K`; the default is a
player. Its callbacks receive `TaskContext<K>` in instance context:

- `owner(...)` runs typed actions as the original owner at its current location.
- `instance()` exposes the instance through `EntityContext<MarkerKind>`.
- `stop()` ends that instance, including from inside its own callbacks.
- `target(...)` traverses a projectile hit entity; block hits have no target.
- `damage_target(...)` applies native damage attributed to the original owner.

Use `owns::<EntityScopedState>()` for per-instance gameplay data. Attachment and
detachment reuse the State lifecycle, including storage cleanup. Other State
scopes are rejected for marker instances. Use `requires::<OwnerStateOrBundle>()`
to end work after an owner leaves an archetype or match. Player State and nested
bundles are supported, as are composed `StateQuery` declarations with optional
and forbidden components. A child `Lifetime<MarkerKind>` started from instance
context is owned by that parent instance; removal of the parent invalidates the
child on its next loaded tick.

Cancellation, expiry, missing/dead owners, required-State removal and reload all
use the same idempotent cleanup path. Cleanup marks an instance as stopping
before calling user code, detaches its State, clears its internal scores and
removes its marker. A callback's early return cannot skip framework cleanup.
Owner traversal can be empty during cleanup after disconnect or disappearance.
Call instance operations such as `stop()` from instance context, outside the
closures passed to `owner` or `target`; those closures temporarily change `@s`.

Owner and required-State checks observe their current state on each loaded
tick. If a player dies and respawns, reconnects, or regains State before the next
observation, those checks alone do not record the intervening transition. The
combat example calls cancellation from canonical `OnDeathEvent` and `OnJoinEvent`
handlers to invalidate work even when its chunk is unloaded. Use the same pattern
for match-leave events when rejoining must not resume old work.

Timers count **loaded ticks**. Sand does not force-load chunks. Unloaded work
observes cancellation or reload invalidation when it loads again. Non-player
owners must remain in the instance's dimension; player lookup supports dimension
changes. The default scan covers the three vanilla dimensions; configure custom
dimensions explicitly. Disabling a datapack prevents its functions from running,
so there is no promise of cleanup while the pack itself is disabled. Re-enabling
and reloading invalidates old instances.

Capacity is per definition, including unloaded instances. IDs never wrap or get
reused. Do not remove private runtime markers through unrelated commands: that
bypasses cleanup and can consume admission capacity until reload.

## Refreshing and stacking statuses

`StatusEffect::new(lifetime)` refreshes an existing owner's duration without
repeating start or stop callbacks. `.stacking(limit)` instead admits independently
expiring stacks up to that owner's limit. Each stack has its own State and
callbacks. `apply()` and `remove()` use the shared lifetime machinery. Refreshes
also reach unloaded instances when they return. Private admission bookkeeping
is removed when the last stack ends; authored status data remains ordinary State.

## System order and transitions

Tick systems can declare typed function-path dependencies:

```rust,ignore
#[system(tick, before = check_deaths)]
fn damage(actors: Combat) { /* ... */ }

#[system(tick, after = damage, every = 2)]
fn check_deaths(actors: Combat) { /* ... */ }
```

Grouped `#[system] impl` methods support `before = Self::other` and
`after = Group::other`. Missing registrations and cycles fail export. Ordering
is over complete entity scans; Sand does not fuse scans in a schedule containing
explicit dependencies. Cadences remain independent: an edge orders systems that
are due, and does not force a skipped predecessor to run. Event dispatch keeps
its own existing event-graph ordering.

A bound canonical `EntityEnum` field also supports
`transition(destination, exit_actions, enter_actions)`. Repeating the current
variant does nothing. Sand commits the new variant before callbacks to prevent
reentrant duplicate exits. If exit changes the variant again, the superseded
enter callback is skipped. Direct `set` remains available for writes without hooks.

## Spatial and numeric behavior

Projectiles sample at intervals no larger than `min(radius, 0.25)` blocks, up to
16 blocks and 256 samples per tick. Each sample
checks passable blocks and at most 16 nearby entity candidates. Owner identities
are excluded from hits. Speed, radius, duration and capacity are validated.
Entity proximity is measured to the entity position, so eye-height casts need
a radius that reaches player feet; the example uses 1.75 blocks. This is sampled
collision rather than a swept hitbox: thin geometry or entities between samples
can be missed. Choose a capacity appropriate to the world's
command budget; bounded sampling does not bound arbitrary user callbacks.

Canonical `StatCurve` additions use checked `/compute` integer arithmetic on
verified 26.3 targets. Older targets keep the checked scoreboard backend.
Rounding, destination bounds, dirty tracking and abort-before-commit behavior
remain shared. Multiplication and division retain their existing implementation.
`StatCurve::distance_squared([x, y, z], [x2, y2, z2])` composes this same model;
compare it to a squared radius when coordinates are stored in numeric State.

## Native gameplay tests

Register `GameplayTest` definitions with `setup`, `after`, `expect` and `cleanup`.
Conditions are ordinary Sand `Condition` values, including canonical target and
State filters. Phases run in server context, so select actors explicitly. A test's
`run()` action schedules it; assertions accumulate failures and cleanup still
runs. The same test cannot overlap itself. Completion prints
`SAND_TEST_PASS <id>` or `SAND_TEST_FAIL <id>`.

These scenarios execute on Minecraft; they do not simulate players or commands
in Rust. The maintained framework proof is `scripts/validate_gameplay.py`, which
exports the public API fixture, starts a disposable server and verifies actual
State, cleanup, collisions, damage and numeric failure behavior. The complementary `scripts/validate_owned_players.py` proof uses two real
players on verified protocol 776 (26.2), including death, disconnect,
reconnection and cancellation across chunk unloading. Player protocol fixtures
remain explicitly versioned. System order and generated function costs are also
visible in the existing runtime report.
