# Events

An `AdvancementEvent` defines a typed advancement trigger; `#[event] fn handler(event: Event<T>)` becomes its reward function. Use `event.player()` as the triggering player.

```rust
pub struct UseApple;
impl AdvancementEvent for UseApple {
 type Trigger = UsingItemTrigger;
 fn trigger() -> Self::Trigger { UsingItemTrigger::new().item(ItemPredicate::id("minecraft:apple")) }
 fn guard() -> Option<Condition> { Some(MANA.of("@s").lt(100)) }
}
#[event] pub fn on_use(event: Event<UseApple>) { MANA.add(event.player(), 5); }
```

The mental model is trigger builder → advancement JSON → reward function. `EntityHurtPlayer`, `PlayerInteractedWithEntityTrigger`, and `SummonedEntityTrigger` are available where vanilla exposes those events. Use tick systems for continuous state or actions with no advancement trigger. Rewards may be revoked/reset according to event configuration.

<div class="sand-warning"><strong>Limit.</strong> Advancement criteria do not expose every runtime detail, including exact event damage amount. Use [Damage Tracking](damage-tracking.md) when approximation is acceptable.</div>
