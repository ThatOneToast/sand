# Item Events

`CustomItemExt` adds typed event and check helpers to `CustomItem`.

```rust
let advancement = shield.on_use_fn(ResourceLocation::new("arcane", "shield/use").unwrap(), shockwave);
shield.item_check_offhand().run(cmd::call(shockwave));
SHIELD.has_in_offhand().run(cmd::call(shockwave));
```

Use `on_use_fn`, `on_kill_fn`, and `on_trigger_fn` with typed function refs. `item_check_in`, mainhand/offhand, and anywhere are execute checks; `CustomItemId` offers equivalent lightweight checks. Advancement timing and available triggers are vanilla-limited; add a scoreboard cooldown deliberately.
