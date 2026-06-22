# Typed Damage

Use typed damage builders instead of raw `/damage` commands.

Direct vanilla damage accepts exactly one entity target:

```rust
cmd::damage(SingleEntity::self_(), 4.0).damageType("minecraft:generic");
```

For many targets, use the high-level builder. Sand lowers this through
`execute as` so Minecraft never sees invalid `damage @e[...]` syntax:

```rust
Damage::new()
    .to(EntityTargets::nearby(5.0).excluding_players().excluding_self())
    .amount(DamageAmount::fixed(4.0))
    .damage_type(DamageKind::Generic)
    .run();
```

Generated command:

```mcfunction
execute as @e[distance=0.1..5,type=!minecraft:player] run damage @s 4 minecraft:generic
```

For reflected damage from an event, Sand centers selection on the damaged
player before looping targets:

```rust
#[event]
pub fn on_damaged(event: DamageEvent<MyDamageEvent>) {
    event
        .reflect_damage()
        .to(EntityTargets::nearby(5.0).excluding_players().excluding_self())
        .amount(DamageAmount::fixed(4.0))
        .damage_type(DamageKind::Generic)
        .run();
}
```

Exact same-as-event damage is not exposed by vanilla advancement rewards. Use
`DamageAmount::fixed(...)` today. `DamageAmount::SameAsEvent` is reserved for a
future tracker-backed implementation.

Many-target source attribution from `@s` is intentionally not claimed yet:
changing executor to each target would make `by @s` refer to the target. Sand
will need a scoped helper/tag or tracker strategy before that is safe.
