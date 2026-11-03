# Combat rules and encounters

Sand supports two explicit health models. Native Minecraft health uses
`player.living().damage(amount, kind)` and retains vanilla armor, immunity and
attribution rules. `Combat` resolves custom health stored in your canonical
State, including shields, resistance, critical hits and healing. It does not
intercept vanilla attacks or silently replace player health.

## Runtime native damage

`LivingEntity::damage` accepts constants, `DamageAmount`, bound numeric State
values and `StatCurve` expressions. Values are sampled in the caller's execution
context. Dynamic amounts use checked arithmetic at thousandth-hit-point
precision. Missing inputs, failed arithmetic and negative runtime results do
no damage; invalid constants fail export. Constants still emit a direct native
command. Dynamic values use a captured function macro argument on every supported
target, including 26.1 and 26.2.

Bind the attacker before selecting victims to preserve native attribution:

```rust,ignore
EntityScope::bind(&attacker, |source| {
    Target::players().within_blocks(6.0).each(|victim| {
        victim.living().damage_by(4.0, DamageKind::Magic, source)
    })
});
```

A scoped source is synchronous and dimension-local. Use the existing owned
projectile context for persistent projectile-owner identity.

## Custom health with canonical State

```rust,ignore
#[derive(State)]
#[state(namespace = "arena", scope = entity)]
struct Fighter {
    #[state(default = 100, min = 0, max = 100)]
    health: Score,
    #[state(default = 20, min = 0, max = 20)]
    shield: Score,
    #[state(auto_tick)]
    invulnerability: Cooldown,
}

fn defenses() -> sand::systems::Combat {
    sand::systems::Combat::new(Fighter::health, 100)
        .shield(Fighter::shield)
        .resistance(0.2)
        .invulnerability(Fighter::invulnerability, Ticks::new(5))
}

// Execute as an entity with Fighter already attached.
defenses().damage(10);
defenses().heal(5);
```

A hit checks positive health and a ready invulnerability cooldown, applies
resistance and an optional `.critical(condition, multiplier)`, consumes shields,
then subtracts remaining damage from health. Resistance is a fraction clamped
to zero through one. Conditions and expressions run in the target context.
Critical conditions with overlapping alternatives still apply the multiplier
once. A hit fully absorbed by a shield counts as accepted and starts
invulnerability. Zero damage and fully resisted hits do not.

Health and shield may be `Score` or `FixedScore`, but must use the same storage
scale and bounds admitting zero. Damage rounds to that scale. All required
calculations succeed before either field or the cooldown changes. Missing
fields reject the operation; combat never attaches State implicitly. Keep
maximum health consistent with the field bounds.

Healing ignores resistance, shields and invulnerability, caps at maximum health,
and does not revive zero health. Revival is an explicit State operation.

`on_hit`, `on_defeat` and `on_heal` append target-context actions. All State
mutations commit before callbacks, so nested hits see current values. A callback
return cannot escape the combat action or its caller. A lethal hit selects the
defeat outcome before callbacks, runs `on_hit`, then `on_defeat`. Callbacks are
notifications for this policy's accepted changes, not global native combat
events. A successful change returns one; rejected or unchanged operations return
zero. There is no persistent transaction entity or separate health registry.

For native mob health integration, use an explicit archetype `HealthBinding` and
choose its synchronization policy. Vanilla player Health NBT cannot be assigned
by this API. Custom player health needs deliberate presentation and native-damage
handling. Healing native health, configurable knockback and global damage
interception are not provided by this combat policy.

## Owned encounter behavior

`Encounter` selects registered child tasks in priority order:

```rust,ignore
Encounter::new(
    Lifetime::new("arena:boss".parse().unwrap(), Ticks::seconds(300))
        .requires::<Fighter>()
        .on_stop(|ctx| ctx.owner(|owner| reset_presentation(owner))),
)
.while_active(|owner| owner.state(Fighter::health).matches(1..).unwrap())
.attack(
    |owner| owner.state(Fighter::health).matches(..=50).unwrap(),
    enraged_attack(),
)
.attack(|_| Condition::all([]), normal_attack())
```

Register the child definitions separately. Selection runs each loaded tick and
starts only the first eligible child when all current direct children have
settled. Put windup, execution and recovery in a child `Sequence`. A rejected
child admission retries that priority choice next tick. It does not spend costs
or select a different attack. At most 256 attack rules are accepted.

`while_active` is checked even during an attack. When false, the parent stops and
its descendants clean up. `start` restarts only the current owner's encounter;
`cancel`, expiry and owner loss use the same Lifetime cleanup. The stop callback
is the place to reset presentation and other State owned by the encounter.
Unloaded children remain outstanding until their ordinary cleanup, so an unloaded
attack cannot silently be replaced with overlapping work.

Conditions can combine phase State, target availability and an attack-turn field.
This is a priority controller, not a threat table, navigation system or behavior
tree. The [animated boss example](../../examples/animated_boss/src/lib.rs) combines
custom shield/health, two native player attacks, health thresholds, telegraphs,
child ownership, recovery, reset and bounded target queries.
