# 7. Item Events

`CustomItemExt` creates an advancement whose reward is a typed function reference.

```rust
let use_wand = wand.on_use_fn(ResourceLocation::new("arcane", "wand/use").unwrap(), cast_spell);
```

Advancement-backed use is not every possible gameplay action; use tick checks where vanilla has no trigger. See [Item Events](../manual/item-events.md).
