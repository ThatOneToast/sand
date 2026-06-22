# Advancement Events

An advancement event type owns three pieces of behavior:

- `trigger()` returns the typed Minecraft advancement trigger.
- `guard()` returns an optional `Condition`.
- `reset()` controls whether the advancement is revoked after firing.

The handler receives `Event<T>`, not `T`:

```rust
#[event]
pub fn on_used_dash_wand(event: Event<UsedDashWandEvent>) {
    MANA.remove(event.player(), 25);
    cmd::call(dash_wand_effect);
}
```

The generated advancement path uses `T: AdvancementEvent` directly, so guards
stay typed as `Option<Condition>` and never flow through legacy string
conditions.

