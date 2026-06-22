# Typed Damage

Direct `/damage` is single-target in vanilla, so Sand models that in Rust:

```rust
cmd::damage(SingleEntity::self_(), 4.0).damageType("minecraft:generic");
```

For many targets, use the high-level builder:

```rust
Damage::new()
    .to(EntityTargets::nearby(5.0).excluding_players().excluding_self())
    .amount(DamageAmount::fixed(4.0))
    .damage_type(DamageKind::Generic)
    .run();
```

Sand lowers that to:

```mcfunction
execute as @e[distance=0.1..5,type=!minecraft:player] run damage @s 4 minecraft:generic
```

Damage events use `DamageEvent<T>`:

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

Exact same-as-event damage is not available from vanilla advancement reward
functions. Use fixed damage unless a real tracker is added.
