# Movement

Enable `systems-movement`. `PushAway`, `Launch`, `SpeedBoost`, and `Slow` generate vanilla teleports/effects.

```rust
PushAway::new().source(Selector::self_()).targets(EntityTargets::nearby(6.0))
 .strength(1.5).lift(0.25).build();
Launch::targets(EntityTargets::nearby(4.0)).amount(0.7).build();
SpeedBoost::target(Selector::self_()).amount(0.4).duration(Ticks::seconds(5)).build();
```

PushAway executes as each target, faces the source, and teleports local negative-Z, locking direction to that source. Narrow selectors in multiplayer. Launch is positional, not physics velocity; arbitrary directional velocity needs significant vanilla score/math work.
