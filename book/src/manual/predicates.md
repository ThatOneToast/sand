# Predicates

`ItemPredicate`, `EntityPredicate`, `EntityEquipment`, and `DamagePredicate` describe Minecraft JSON conditions, not command selectors.

```rust
let item = ItemPredicate::id("minecraft:shield");
let entity = EntityPredicate::new().nbt("{Tags:[\"altar\"]}");
```

Use them in advancement triggers and component JSON. `DamagePredicate` can express damage criterion filters; it does not recover numerical runtime damage. Use raw predicates only for unmodeled Minecraft-version fields.
