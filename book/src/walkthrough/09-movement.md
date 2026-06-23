# 9. Movement And Shockwaves

Enable `systems-movement` and produce a shield shockwave:

```rust
PushAway::new().source(Selector::self_())
 .targets(EntityTargets::nearby(6.0).excluding_players()).strength(1.5).lift(0.25).build();
```

This is a local-coordinate teleport displacement, not true velocity. See [Movement](../manual/movement.md).
