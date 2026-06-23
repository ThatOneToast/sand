# Conditions

`Condition` represents score, predicate, entity, and storage checks, then lowers into valid execute clauses. Compose with `all!`, `any!`, `when`, and `unless`.

```rust
when(all![MANA.of("@s").gte(20), HAS_WAND.of("@s").is_true()]).then(cmd::say("ready"));
```

`any!` may lower into multiple commands because Minecraft execute has no general boolean OR. This is expected. Use item checks through `InventorySystem` and entity/item/damage JSON matching through [Predicates](predicates.md).
